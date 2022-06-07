use std::collections::HashMap;

use urlencoding::decode;

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
  pub file: HashMap<String, String>,    // FILE data
  pub cookie: HashMap<String, String>,  // Cookies
}

impl Request {
  // Constructor
  pub fn new(param: &HashMap<String, String>, dir: String) -> Request {
    let mut get = HashMap::with_capacity(16);
    let mut post = HashMap::with_capacity(128);
    let mut file = HashMap::with_capacity(128);
    let mut cookie = HashMap::with_capacity(16);

    let key = "HTTP_X_REQUESTED_WITH".to_string();
    let ajax = if param.contains_key(&key) && param.get(&key).unwrap().to_lowercase().eq(&"xmlhttprequest".to_string()) { true } else { false };
    let key = "HTTP_HOST".to_string();
    let host = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_string() };
    let key = "REQUEST_SCHEME".to_string();
    let scheme = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "https".to_string() };
    let key = "HTTP_USER_AGENT".to_string();
    let agent = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_string() };
    let key = "HTTP_REFERER".to_string();
    let referer = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_string() };
    let key = "REMOTE_ADDR".to_string();
    let ip = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_string() };
    let key = "REQUEST_METHOD".to_string();
    let method = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_string() };
    let key = "REDIRECT_URL".to_string();
    let url = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { "".to_string() };
    let url = decode(&url.splitn(2, '?').next().unwrap().to_string()).unwrap_or_default().to_string();
    let key = "DOCUMENT_ROOT".to_string();
    let path = if param.contains_key(&key) { param.get(&key).unwrap().to_owned() } else { dir };
    let site = format!("{}://{}", scheme, host);
    let key = "QUERY_STRING".to_string();
    if param.contains_key(&key) {
      let val = param.get(&key).unwrap();
      let gets:Vec<&str> = val.split("&").collect();
      for v in gets {
        let key: Vec<&str> = v.splitn(2, "=").collect();
        match key.len() {
          1 => get.insert(decode(v).unwrap_or_default().to_string(), "".to_string()),
          _ => get.insert(decode(key[0]).unwrap_or_default().to_string(), decode(key[1]).unwrap_or_default().to_string()),
        };
      }
    }
    let key = "HTTP_COOKIE".to_string();
    if param.contains_key(&key) {
      let val = param.get(&key).unwrap();
      let cooks:Vec<&str> = val.split("; ").collect();
      for v in cooks {
        let key: Vec<&str> = v.splitn(2, "=").collect();
        if key.len() == 2 {
          cookie.insert(key[0].to_string(), key[1].to_string());
        }
      }
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
}