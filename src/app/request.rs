use std::{collections::HashMap, io::Write};
use urlencoding::decode;
use tempfile::NamedTempFile;

// Loaded file
#[derive(Debug)]
pub struct WebFile {
  pub size: usize,                      // File size
  pub name: String,                     // File name
  pub tmp: String,                      // Absolute path to file location
}

// Request from client
pub struct Request {
  pub ajax: bool,                       // Ajax query (only software detect)
  pub host: String,                     // Request host. Example: subdomain.domain.zone
  pub scheme: String,                   // Request scheme. Example: http / https
  pub agent: String,                    // HTTP_USER_AGENT
  pub referer: String,                  // HTTP_REFERER
  pub ip: String,                       // Client IP
  pub method: String,                   // REQUEST_METHOD
  pub path: String,                     // DOCUMENT_ROOT
  pub site: String,                     // Request site. Example: https://subdomain.domain.zone
  pub url: String,                      // Request url. Example: /product/view/item/145
  pub get: HashMap<String, String>,     // GET data
  pub post: HashMap<String, String>,    // POST data
  pub file: HashMap<String, Vec<WebFile>>, // FILE data
  pub cookie: HashMap<String, String>,  // Cookies
}

impl Request {
  // Constructor
  pub fn new(param: &HashMap<String, String>, stdin: &Option<Vec<u8>>, dir: &str) -> Request {
    let mut get = HashMap::with_capacity(16);
    let mut post = HashMap::with_capacity(128);
    let mut file = HashMap::with_capacity(16);
    let mut cookie = HashMap::with_capacity(16);

    let key = "HTTP_X_REQUESTED_WITH".to_owned();
    let ajax = if param.contains_key(&key) && param.get(&key).unwrap().to_lowercase().eq(&"xmlhttprequest".to_owned()) { true } else { false };
    let key = "HTTP_HOST".to_owned();
    let host = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_owned() };
    let key = "REQUEST_SCHEME".to_owned();
    let scheme = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "https".to_owned() };
    let key = "HTTP_USER_AGENT".to_owned();
    let agent = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_owned() };
    let key = "HTTP_REFERER".to_owned();
    let referer = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_owned() };
    let key = "REMOTE_ADDR".to_owned();
    let ip = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_owned() };
    let key = "REQUEST_METHOD".to_owned();
    let method = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_owned() };
    let key = "REDIRECT_URL".to_owned();
    let url = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_owned() };
    let url = decode(&url.splitn(2, '?').next().unwrap().to_owned()).unwrap_or_default().to_string();
    let key = "DOCUMENT_ROOT".to_owned();
    let path = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { dir.to_owned() };
    let site = format!("{}://{}", scheme, host);

    // Extract GET data 
    let key = "QUERY_STRING".to_owned();
    if param.contains_key(&key) {
      let val = param.get(&key).unwrap();
      let gets:Vec<&str> = val.split("&").collect();
      for v in gets {
        let key: Vec<&str> = v.splitn(2, "=").collect();
        match key.len() {
          1 => get.insert(decode(v).unwrap_or_default().to_string(), "".to_owned()),
          _ => get.insert(decode(key[0]).unwrap_or_default().to_string(), decode(key[1]).unwrap_or_default().to_string()),
        };
      }
    }
    // Extract COOKIE data 
    let key = "HTTP_COOKIE".to_owned();
    if param.contains_key(&key) {
      let val = param.get(&key).unwrap();
      let cooks:Vec<&str> = val.split("; ").collect();
      for v in cooks {
        let key: Vec<&str> = v.splitn(2, "=").collect();
        if key.len() == 2 {
          cookie.insert(key[0].to_owned(), key[1].to_owned());
        }
      }
    }
    // Extract POST data 
    let key = "CONTENT_TYPE".to_owned();
    let content = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_owned() };
    if content.len() > 0 {
      if let "application/x-www-form-urlencoded" = &content[..] {
        //Simple post
        if let Some(data) = stdin {
          if let Ok(v) = String::from_utf8(data.to_owned()) {
            let val: Vec<&str> = v.split("&").collect();
            for v in val {
              let val: Vec<&str> = v.splitn(2, "=").collect();
              match val.len() {
                1 => post.insert(decode(v).unwrap_or_default().to_string(), "".to_owned()),
                _ => post.insert(decode(val[0]).unwrap_or_default().to_string(), decode(val[1]).unwrap_or_default().to_string()),
              };
            }
          };
        };
      } else if let "multipart/form-data; boundary=" = &content[..30] {
        // Multi post with files
        let boundary = format!("--{}", &content[30..]).as_bytes().to_vec();
        let stop: [u8; 4] = [13, 10, 13, 10];
        if let Some(data) = stdin {
          let mut seek: usize = 0;
          let mut start: usize;
          let b_len = boundary.len();
          let len = data.len() - 4;
          let mut found: Option<(usize, String)> = None;
          while seek < len {
            // Find a boundary
            if boundary == data[seek..seek + b_len] {
              if seek + b_len == len {
                if let Some((l, h)) = found {
                  let d = &data[l..seek - 2];
                  Request::get_post_file(h, d, &mut post, &mut file);
                };
                break;
              }
              seek += b_len + 2;
              start = seek;
              while seek < len {
                if stop == data[seek..seek+4] {
                  if let Ok(s) = String::from_utf8((&data[start..seek]).to_owned()) {
                    if let Some((l, h)) = found {
                      let d = &data[l..start-b_len-4];
                      Request::get_post_file(h, d, &mut post, &mut file);
                    };
                    found = Some((seek+4, s));
                  }
                  seek += 4;
                  break;
                } else {
                  seek += 1;
                }
              }
            } else {
              seek += 1;
            }
          }
        };
      };
    }

    Request{
      ajax,
      host,
      scheme,
      agent,
      referer,
      ip,
      method,
      path,
      site,
      url,
      get,
      post,
      cookie,
      file,
    }
  }

  // get post file from multipart/form-data
  fn get_post_file(header: String, data: &[u8], post: &mut HashMap<String, String>, file: &mut HashMap<String, Vec<WebFile>>) {
    let h: Vec<&str> = header.splitn(3, "; ").collect();
    let len = h.len();
    if len == 2 {
      if let Ok(v) = String::from_utf8(data.to_vec()) {
        let k = h[1][6..h[1].len() - 1].to_owned();
        post.insert(k, v);
      }
    } else if len == 3 {
      let k = h[1][6..h[1].len() - 1].to_owned();
      let n: Vec<&str> = h[2].splitn(2, "\r\n").collect();
      let n = n[0][10..n[0].len()-1].to_owned();

      if let Ok(tmp) = NamedTempFile::new() {
        if let Ok((mut f, p)) = tmp.keep() {
          if let Some(path) = p.to_str() {
            if let Ok(_) = f.write_all(data) {
              if let None = file.get(&k) {
                file.insert(k.clone(), Vec::with_capacity(16));
              }
              file.get_mut(&k).unwrap().push(WebFile { size: data.len(), name: n, tmp: path.to_owned()});
            }
          }
        }
      }
    }
  }
}