use std::{sync::{Mutex, Arc}, rc::Rc, cell::RefCell, collections::HashMap, io::Write};

use postgres::{Client, Row};
use postgres_protocol::escape::escape_literal;
use cast::{u8, u64};
use urlencoding::decode;
use tempfile::NamedTempFile;
use chrono::Local;
use regex::Regex;
use serde_json::{Value, Map, Number};
use sha3::{Digest, Sha3_512};

use crate::{sys::go::{storage::Storage, i18n::LangItem}};

pub const ON_YEAR: u32 = 31622400;
pub const DEFAULT_LANG: u8 = 0;

// The type of response from the controller
pub enum Answer{
  None,               // With out answer
  String(String),     // Answer in the form of text
  // Raw(Vec<u8>),       // Answer in binary data
}

// Type of data, which use in server
pub enum Data {
  None,
  U8(u8),
  I64(i64),
  U64(u64),
  F64(f64),
  Bool(bool),
  String(String),
  Vec(Vec<Data>),
  VecLang((u8, Vec<LangItem>)),
  Map(HashMap<String, Data>),       // Map of string keys
}

// Loaded file
pub struct WebFile {
  pub size: usize,                      // File size
  pub name: String,                     // File name
  pub tmp: String,                      // Absolute path to file location
}

// Redirect header (HTTP Location)
pub struct Location {
  pub url: String,              // Url
  pub permanently: bool,        // Permanently redirect
}

// Cookie struct
pub struct Cookie {
  pub key: String,              // Session key
  pub value: String,            // Session value
  pub time: u32,                // Max-Age cookies value
}

// Main CRM struct
pub struct Action {
  pub module: String,                       // Run module
  pub class: String,                        // Run class
  pub action: String,                       // Run controller
  pub salt: String,                         // Salt for password

  db_sql: Rc<RefCell<Client>>,              // Postgresql connection
  pub db_counts: u64,                       // Count selected data
  pub db_err: bool,                         // Error of sql query
  pub db_query: String,                     // Last sql text
  pub db_error: String,                     // Error text

  storage: Arc<Mutex<Storage>>,             // Global cache

  pub ajax: bool,                           // Ajax query (only software detect)
  pub host: String,                         // Request host. Example: subdomain.domain.zone
  pub scheme: String,                       // Request scheme. Example: http / https
  pub agent: String,                        // HTTP_USER_AGENT
  pub referer: String,                      // HTTP_REFERER
  pub ip: String,                           // Client IP
  pub method: String,                       // REQUEST_METHOD
  pub path: String,                         // DOCUMENT_ROOT
  pub site: String,                         // Request site. Example: https://subdomain.domain.zone
  pub url: String,                          // Request url. Example: /product/view/item/145
  pub get: HashMap<String, String>,         // GET data
  pub post: HashMap<String, String>,        // POST data
  pub file: HashMap<String, Vec<WebFile>>,  // FILE data
  pub cookie: HashMap<String, String>,      // Cookies

  pub set_cookie: Cookie,                   // Cookie
  location: Option<Location>,               // Redirect (HTTP Location)
  pub http_code: Option<u16>,               // Header code (HTTP code)
  pub css: Vec<String>,                     // Addition css script
  pub js: Vec<String>,                      // Addition js script
  pub lang_code: String,                    // Current language code

  pub user_id: i64,                         // user_id from database
  pub session_id: i64,                      // session_id from database
  pub session: String,                      // cookie key
  session_data: HashMap<String, Data>,      // User data
  session_change: bool,                     // User data is changed

  pub lang_id: u8,                                                                                              // User lang_id 
  i18n: Rc<RefCell<HashMap<u8, HashMap<String, HashMap<String, Rc<RefCell<HashMap<String, String>>>>>>>>,       // Global ref to tranlations
  data: Option<Rc<RefCell<HashMap<String, String>>>>,                                                           // Local copy of translation for web controller
  langs: Rc<RefCell<HashMap<u8, LangItem>>>,                                                                    // List of enable langs
  sort: Rc<RefCell<Vec<LangItem>>>,                                                                             // Sorted list of langs
}

impl Action {
  // Constructor
  pub fn new(
    sql: Rc<RefCell<Client>>, 
    salt: String, 
    storage: Arc<Mutex<Storage>>, 
    i18n: Rc<RefCell<HashMap<u8, HashMap<String, HashMap<String, Rc<RefCell<HashMap<String, String>>>>>>>>, 
    param: &HashMap<String, String>, 
    stdin: &Option<Vec<u8>>, 
    dir: String,
    langs: Rc<RefCell<HashMap<u8, LangItem>>>,
    sort: Rc<RefCell<Vec<LangItem>>>,
  ) -> Action {

    let module = "".to_owned();
    let class = "".to_owned();
    let action = "".to_owned();

    // Request init
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
                  Action::get_post_file(h, d, &mut post, &mut file);
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
                      Action::get_post_file(h, d, &mut post, &mut file);
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

    // session init
    let mut session_change = false;
    let session_data = HashMap::new();
    let cook_key = "tryteex".to_owned();

    // Get and check cookie
    let mut session = match cookie.get(&cook_key) {
      Some(s) => {
        match Regex::new(r"^[a-f0-9]{128}$") {
          Ok(rx) => {
            if rx.is_match(s) {
              s.to_owned()
            } else {
              "".to_owned()
            }
          },
          Err(_) => "".to_owned(),
        }
      },
      None => "".to_owned(),
    };
    if session.len() == 0 {
      // Generate a new cookie
      let time = Local::now().format("%Y.%m.%d %H:%M:%S%.9f %:z").to_string();
      let cook = format!("{}{}{}{}{}", salt, ip, agent, host, time);
      let mut hasher = Sha3_512::new();
      hasher.update(cook.as_bytes());
      session = format!("{:#x}", hasher.finalize());
      session_change = true;
    }

    let act = Action {
      module,
      class,
      action,
      salt,

      // db
      db_sql: sql,
      db_counts: 0,
      db_err: false,
      db_query: String::with_capacity(65536),
      db_error: String::with_capacity(65536),

      // cache
      storage,

      // request
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

      // response
      http_code: None,
      set_cookie: Cookie { key: cook_key, value: session.clone(), time: ON_YEAR },
      location: None,
      css: Vec::with_capacity(16),
      js: Vec::with_capacity(16),
      lang_code: "".to_owned(),

      // seccion
      user_id: 0,
      session_id: 0,
      session,
      session_data,
      session_change,

      // lang
      lang_id: DEFAULT_LANG,
      i18n,
      data: None,
      langs,
      sort,
    };
    
    // Load the user data
    act.session_load();

    act
  }

  // DB block
  // Execute query to database
  pub fn db_query(&mut self, sql: &String)->Vec<Row> {
    let mut db = self.db_sql.borrow_mut();
    
    match db.query(sql, &[]) {
      Ok(res) => {
        self.db_err = false;
        self.db_counts = u64(res.len());
        self.db_error = "".to_owned();
        self.db_query = sql.to_owned();
        return res;
      },
      Err(e) => {
        self.db_err = true;
        self.db_counts = 0;
        self.db_error = e.to_string();
        self.db_query = sql.to_owned();
        return Vec::new();
      },
    };
  }

  // Escape text.
  pub fn db_escape(&self, text: &str)->String {
    escape_literal(&text)
  }

  // Cache block
  // Set value
  pub fn cache_set(&mut self, key: String, value: Data) {
    let mut s = Mutex::lock(&self.storage).unwrap();
    s.set(key, value);
  }
  
  // Get value
  pub fn cache_get(&self, key: &str) -> Option<Data> {
    let storage = Mutex::lock(&self.storage).unwrap();
    match storage.get(key) {
      Some(data) => {
        let val = Mutex::lock(&data).unwrap();
        Some(self.cache_set_value(&val))
      },
      None => None,
    }
  }
  
  // Decode value
  fn cache_set_value(&self, value: &Data) -> Data {
    match value {
      Data::U8(v) => Data::U8(*v),
      Data::I64(v) => Data::I64(*v),
      Data::U64(v) => Data::U64(*v),
      Data::F64(v) => Data::F64(*v),
      Data::Bool(v) => Data::Bool(*v),
      Data::String(v) => Data::String(v.clone()),
      Data::Vec(v) => {
        let mut val: Vec<Data> = Vec::with_capacity(v.len());
        for vl in v {
          val.push(self.cache_set_value(&vl));
        }
        Data::Vec(val)
      },
      Data::Map(v) => {
        let mut val: HashMap<String, Data> = HashMap::with_capacity(v.len());
        for (key, vl) in v {
          val.insert(key.clone(), self.cache_set_value(&vl));
        }
        Data::Map(val)
      },
      Data::None | Data::VecLang(_) => Data::None,
    }
  }

  // Setting block
  // Getting setting
  pub fn setting_get(&mut self, key: &str) -> Option<String> {
    let cache_key = format!("setting:{}", key);
    // Check cache
    if let Some(data) = self.cache_get(&cache_key) {
      if let Data::String(val) = data {
        return Some(val.clone());
      }
    }
    // Read database
    let sql = format!("
      SELECT data
      FROM 
        setting 
      WHERE key={}
      ", self.db_escape(key));
    let res = self.db_query(&sql);
    if res.len() == 0 {
      return None;
    }
    let row = &res[0];
    let value: String = row.get(0);
    self.cache_set(cache_key, Data::String(value.clone()));
    Some(value)
  }

  // Request block
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

  // Set redirect
  pub fn set_redirect(&mut self, url: String, permanently: bool) {
    self.location = Some(Location {url: format!("Location: {}", url), permanently, });
  }

  // Get reditrect
  pub fn get_redirect(&self) -> Option<&Location> {
    self.location.as_ref()
  }

  // Get text from http code
  pub fn get_code(code: u16) -> String {
    match code {
      100 => format!("{} Continue", code),
      101 => format!("{} Switching Protocols", code),
      102 => format!("{} Processing", code),
      103 => format!("{} Early Hints", code),
      200 => format!("{} OK", code),
      201 => format!("{} Created", code),
      202 => format!("{} Accepted", code),
      203 => format!("{} Non-Authoritative Information", code),
      204 => format!("{} No Content", code),
      205 => format!("{} Reset Content", code),
      206 => format!("{} Partial Content", code),
      207 => format!("{} Multi-Status", code),
      208 => format!("{} Already Reported", code),
      226 => format!("{} IM Used", code),
      300 => format!("{} Multiple Choices", code),
      301 => format!("{} Moved Permanently", code),
      302 => format!("{} Found", code),
      303 => format!("{} See Other", code),
      304 => format!("{} Not Modified", code),
      305 => format!("{} Use Proxy", code),
      306 => format!("{} (Unused)", code),
      307 => format!("{} Temporary Redirect", code),
      308 => format!("{} Permanent Redirect", code),
      400 => format!("{} Bad Request", code),
      401 => format!("{} Unauthorized", code),
      402 => format!("{} Payment Required", code),
      403 => format!("{} Forbidden", code),
      404 => format!("{} Not Found", code),
      405 => format!("{} Method Not Allowed", code),
      406 => format!("{} Not Acceptable", code),
      407 => format!("{} Proxy Authentication Required", code),
      408 => format!("{} Request Timeout", code),
      409 => format!("{} Conflict", code),
      410 => format!("{} Gone", code),
      411 => format!("{} Length Required", code),
      412 => format!("{} Precondition Failed", code),
      413 => format!("{} Content Too Large", code),
      414 => format!("{} URI Too Long", code),
      415 => format!("{} Unsupported Media Type", code),
      416 => format!("{} Range Not Satisfiable", code),
      417 => format!("{} Expectation Failed", code),
      418 => format!("{} (Unused)", code),
      421 => format!("{} Misdirected Request", code),
      422 => format!("{} Unprocessable Content", code),
      423 => format!("{} Locked", code),
      424 => format!("{} Failed Dependency", code),
      425 => format!("{} Too Early", code),
      426 => format!("{} Upgrade Required", code),
      428 => format!("{} Precondition Required", code),
      429 => format!("{} Too Many Requests", code),
      431 => format!("{} Request Header Fields Too Large", code),
      451 => format!("{} Unavailable For Legal Reasons", code),
      500 => format!("{} Internal Server Error", code),
      501 => format!("{} Not Implemented", code),
      502 => format!("{} Bad Gateway", code),
      503 => format!("{} Service Unavailable", code),
      504 => format!("{} Gateway Timeout", code),
      505 => format!("{} HTTP Version Not Supported", code),
      506 => format!("{} Variant Also Negotiates", code),
      507 => format!("{} Insufficient Storage", code),
      508 => format!("{} Loop Detected", code),
      510 => format!("{} Not Extended (OBSOLETED)", code),
      511 => format!("{} Network Authentication Required", code),
      code => format!("{} Unassigned", code),
    }
  }

  // Session block
  // Get user lang_id
  pub fn get_lang_id(&self) -> Option<u8> {
    let key = "lang_id".to_owned();
    match self.session_get(&key) {
        Some(d) => match d {
          Data::U8(v) => Some(*v),
          Data::I64(v) => Some(u8(*v).unwrap()),
          _ => None
        },
        None => None,
    }
  }

  // Set a session data
  pub fn session_set(&mut self, key: String, val: Data) {
    self.session_data.insert(key, val);
    self.session_change = true;
  }

  // Get a session date
  pub fn session_get(&self, key: &String) -> Option<&Data> {
    self.session_data.get(key)
  }

  // Load user session date from database 
  fn session_load(&mut self) {
    let ses_esc = self.db_escape(&self.session);
    let sql = format!("
      WITH 
        new_q AS (
          SELECT 0::int8 user_id, {}::text session, '{{}}'::jsonb data, now() created, now() last, {} ip, {} user_agent
        ),
        ins_q AS (
          INSERT INTO session (user_id, session, data, created, last, ip, user_agent) 
          SELECT n.user_id, n.session, n.data, n.created, n.last, n.ip, n.user_agent
          FROM 
            new_q n
            LEFT JOIN session s ON s.session=n.session
          WHERE s.session_id IS NULL
          RETURNING session_id, data, user_id
        )
      SELECT session_id, data::text, user_id FROM ins_q
      UNION 
      SELECT session_id, data::text, user_id FROM session WHERE session={}
    ", ses_esc, self.db_escape(&self.ip), self.db_escape(&self.agent), ses_esc);
    let res = self.db_query(&sql);
    if res.len() != 1 {
      return;
    }
    let row = &res[0];
    let session_id: i64 = row.get(0);
    let user_id: i64 = row.get(2);
    let data: String = row.get(1);

    self.session_id = session_id;
    self.user_id = user_id;
    let json: Value = serde_json::from_str(&data).unwrap();
    if let Value::Object(obj) = json {
      for (key, val) in obj {
        self.session_data.insert(key, self.session_get_value(val));
      }
    };
  }

  // Decode user date from json
  fn session_get_value(&self, val: Value) -> Data {
    match val {
      Value::Null => Data::None,
      Value::Bool(v) => Data::Bool(v),
      Value::Number(v) => {
        if v.is_i64() { Data::I64(v.as_i64().unwrap()) }
        else if v.is_u64() { Data::U64(v.as_u64().unwrap()) }
        else { Data::F64(v.as_f64().unwrap()) }
      },
      Value::String(v) => Data::String(v),
      Value::Array(v) => {
        let mut vec: Vec<Data> = Vec::with_capacity(v.len());
        for vl in v {
          vec.push(self.session_get_value(vl));
        }
        Data::Vec(vec)
      },
      Value::Object(v) => {
        let mut map: HashMap<String, Data> = HashMap::with_capacity(v.len());
        for (k, vl) in v {
          map.insert(k, self.session_get_value(vl));
        }
        Data::Map(map)
      },
    }
  }

  // Encode user data to json
  fn session_set_value(&self, val: &Data) -> Value {
    match val {
      Data::None | Data::VecLang(_) => Value::Null,
      Data::U8(v) => Value::Number(Number::from(*v)),
      Data::I64(v) => Value::Number(Number::from(*v)),
      Data::U64(v) => Value::Number(Number::from(*v)),
      Data::F64(v) => Value::Number(Number::from_f64(*v).unwrap()),
      Data::Bool(v) => Value::Bool(*v),
      Data::String(v) => Value::String(v.clone()),
      Data::Vec(v) => {
        let mut val: Vec<Value> = Vec::with_capacity(v.len());
        for vl in v {
          val.push(self.session_set_value(vl));
        }
        Value::Array(val)
      },
      Data::Map(v) => {
        let mut val: Map<String, Value> = Map::with_capacity(v.len());
        for (key, vl) in v {
          val.insert(key.to_owned(), self.session_set_value(vl));
        }
        Value::Object(val)
      },
    }
  }

  // Save session date to database
  pub fn session_save(&mut self) {
    if self.session_change {
      // if data were chaged then save it
      let mut map = Map::with_capacity(self.session_data.len());
      for (key, val) in &self.session_data {
        map.insert(key.clone(), self.session_set_value(val));
      };
      let data = serde_json::to_string(&Value::Object(map)).unwrap();
      let sql = format!("
        UPDATE session 
        SET 
          user_id={}, 
          data = {},
          last = now(),
          ip = {},
          user_agent = {}
        WHERE
          session_id={}
      ", self.user_id, data, self.db_escape(&self.ip), self.db_escape(&self.agent), self.session_id);
      self.db_query(&sql);
    } else {
      // If date not changed, only update last visit time
      let sql = format!("
        UPDATE session 
        SET 
          last = now()
        WHERE
          session_id={}
      ", self.session_id);
      self.db_query(&sql);
    }
  }

  // Auth block
  // Checking access to the web controller
  pub fn get_access(&mut self, module: &str, class: &str, action: &str) -> bool {
    // System user always has access
    let key = "system".to_owned();
    if let Some(system) = self.session_get(&key) {
      if let Data::Bool(v) = system {
        if *v { return true; };
      }
    }

    let key = format!("auth:{}:{}:{}:{}", self.user_id, module, class, action);
    // Check access in cache
    if let Some(a) = self.cache_get(&key) {
      if let Data::Bool(val) = a {
        return val;
      } else {
        return false;
      }
    }

    // Prepare sql query
    let mut w: Vec<String> = Vec::with_capacity(4);
    w.push("(c.module='' AND c.class='' AND c.action='')".to_owned());
    if module.len() > 0 {
      w.push(format!("(c.module='{}' AND c.class='' AND c.action='')", module));
      if class.len() > 0 {
        w.push(format!("(c.module='{}' AND c.class='{}' AND c.action='')", module, class));
        if action.len() > 0 {
          w.push(format!("(c.module='{}' AND c.class='{}' AND c.action='{}')", module, class, action));
        }
      }
    }

    let sql = format!("
      SELECT COALESCE(MAX(a.access::int), 0) AS access
      FROM 
        access a
        INNER JOIN user_role u ON u.role_id=a.role_id
        INNER JOIN controller c ON a.controller_id=c.controller_id
      WHERE a.access AND u.user_id={} AND ({})
    ", self.user_id, w.join(" OR ").to_owned());
    let res = self.db_query(&sql);
    if res.len() == 1 {
      let row = &res[0];
      let access: i32 = row.get(0);
      if access == 1 {
        self.cache_set(key, Data::Bool(true));
        return true;
      }
    }
    self.cache_set(key, Data::Bool(false));
    false
  }

  // Main block
  // Start CRM system
  pub fn start(&mut self) -> Answer {
    // Encode routes
    if let Some((module, class, action, params, lang_id)) = self.extract_route() {
      self.set_lang_id(lang_id);
      self.lang_code = self.lang_get_code();
      let mut data: HashMap<String, Data> = HashMap::with_capacity(256);
      // Start CRM system with fixed struct
      return self.start_route(&module, &class, &action, &params, &mut data, false);
    }
    return Answer::None;
  }

  // Encode routes
  fn extract_route(&mut self) -> Option<(String, String, String, String, Option<u8>)> {

    // Find redirect
    let url = self.db_escape(&self.url);
    let key = format!("redirect:{}", &self.url);
    if let Some(data) = self.cache_get(&key) {
      if let Data::String(r) = data {
        let permanently = if r.starts_with("1") { true } else { false };
        self.set_redirect(r[1..].to_owned(), permanently);
        return None;
      }
    }
    let sql = format!("
      SELECT redirect, permanently
      FROM redirect
      WHERE url={}
    ", url);
    let res = self.db_query(&sql);
    if res.len() == 1 {
      let row = &res[0];
      let redirect: String = row.get(0);
      let code: bool = row.get(1);
      let c = if code { "1" } else { "0" };
      let permanently = if code { true } else { false };
      let value = format!("{}{}", c, redirect);
      self.set_redirect(redirect, permanently);
      self.cache_set(key, Data::String(value));
      return None;
    }
    self.cache_set(key, Data::None);

    // Get route
    let key = format!("route:{}", &self.url);
    if let Some(data) = self.cache_get(&key) {
      if let Data::String(r) = data {
        let res: Vec<&str> = r.splitn(5, ":").collect();
        let module = res[0].to_owned();
        let class = res[1].to_owned();
        let action = res[2].to_owned();
        let params = res[3].to_owned();
        let lang_id = res[4].parse::<u8>().unwrap();
        return Some((module, class, action, params, Some(lang_id)));
      }
    } 
    let sql = format!("
      SELECT c.module, c.class, c.action, r.params, r.lang_id
      FROM route r INNER JOIN controller c ON r.controller_id=c.controller_id
      WHERE 
        r.url={} AND LENGTH(c.module)>0 AND LENGTH(c.class)>0 AND LENGTH(c.action)>0
    ", url);
    let res = self.db_query(&sql);
    if res.len() == 1 {
      let row = &res[0];
      let module: String = row.get(0);
      let class: String = row.get(1);
      let action: String = row.get(2);
      let params: String = row.get(3);
      let lang_id: i64 = row.get(4);
      let lang_id = u8(lang_id).unwrap();
      let value = format!("{}:{}:{}:{}:{}", module, class, action, params, lang_id.to_owned());
      self.cache_set(key, Data::String(value));
      return Some((module, class, action, params, Some(lang_id)));
    }
    self.cache_set(key, Data::None);

    // Encode route
    let mut module = "index".to_owned();
    let mut class = "index".to_owned();
    let mut action = "index".to_owned();
    let mut params = "index".to_owned();
    if &self.url != "/" {
      let load: Vec<&str> = self.url.splitn(5, "/").collect();
      let len = load.len();
      if len == 2{
        module = load[1].to_owned();
      } else if len == 3 {
        module = load[1].to_owned();
        class = load[2].to_owned();
      } else if len == 4 {
        module = load[1].to_owned();
        class = load[2].to_owned();
        action = load[3].to_owned();
      } else if len == 5 {
        module = load[1].to_owned();
        class = load[2].to_owned();
        action = load[3].to_owned();
        params = load[4].to_owned();
      }
    }
    Some((module, class, action, params, None))
  }

  // Stop server
  pub fn stop(&mut self) {
    self.session_save();
  }

  // Start CRM system with fixed struct
  fn start_route(&mut self, module: &str, class: &str, action: &str, params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    // Get Access
    let access = self.get_access(module, class, action);

    if access {
      // Run controller
      return self.run(module, class, action, params, data, internal);
    }
    
    // Not found
    if internal {
      return Answer::String("not_found".to_owned());
    }
    self.set_redirect("/index/index/not_found".to_owned(), false);
    Answer::None
  }

  // Load internal controller
  pub fn load(&mut self, module: &str, class: &str, action: &str, params: &str, data: &mut HashMap<String, Data>) -> Answer {
    self.start_route(module, class, action, params, data, true)
  }

  // Run controller
  fn run (&mut self, module: &str, class: &str, action: &str, params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    self.module = module.to_owned();
    self.class = class.to_owned();
    self.action = action.to_owned();
    match module {
      "admin" => match class {
        "index" => {
          let mut app = super::admin::index::App::new(self);
            match action {
            "index" => return app.index(params, data, internal),
            "main" => return app.main(params, data, internal),
            _ => {}
          };
        },
        _ => {},
      },
      "index" => match class {
        "index" => {
          let mut app = super::index::index::App::new(self);
            match action {
            "index" => return app.index(params, data, internal),
            "head" => return app.head(params, data, internal),
            "foot" => return app.foot(params, data, internal),
            "not_found" => return app.not_found(params, data, internal),
            _ => {}
          };
        },
        "cart" => {
          let mut app = super::index::cart::App::new(self);
            match action {
            "index" => return app.index(params, data, internal),
            _ => {}
          };
        },
        "menu" => {
          let mut app = super::index::menu::App::new(self);
            match action {
            "header" => return app.header(params, data, internal),
            "products" => return app.products(params, data, internal),
            "list" => return app.list(params, data, internal),
            "logo" => return app.logo(params, data, internal),
            "upper" => return app.upper(params, data, internal),
            _ => {}
          };
        },
        "search" => {
          let mut app = super::index::search::App::new(self);
            match action {
            "main" => return app.main(params, data, internal),
            "small" => return app.small(params, data, internal),
            _ => {}
          };
        },
        _ => {},
      },
      "user" => match class {
        "admin" => {
          let mut app = super::user::admin::App::new(self);
            match action {
            "index" => return app.index(params, data, internal),
            _ => {}
          };
        },
        "index" => {
          let mut app = super::user::index::App::new(self);
            match action {
            "menu" => return app.menu(params, data, internal),
            "up" => return app.up(params, data, internal),
            _ => {}
          };
        },
        _ => {},
      },
      _ => {},
    };
    Answer::None
  }

  // Lang block
  // Get the correct default user language
  pub fn set_lang_id(&mut self, lang_id: Option<u8>) {
    let key = "lang_id".to_owned();
    match lang_id {
      None => match self.get_lang_id() {
        Some(lang_id) => self.lang_id = lang_id,
        None => {
          let lang_id = DEFAULT_LANG;
          self.session_set(key, Data::U8(lang_id));
          self.lang_id = lang_id;
        },
      },
      Some(lang_id) => match self.get_lang_id() {
        Some(s_lang_id) => {
          let l_id = s_lang_id;
          if l_id != lang_id {
            session.set(key, Data::U8(lang_id));
          }
          self.lang_id = lang_id
        },
        None => {
          session.set(key, Data::U8(lang_id));
          self.lang_id = lang_id
        },
      },
    }
  }

  // Get current language code
  pub fn lang_get_code(&self) -> String {
    self.langs.borrow().get(&self.lang_id).unwrap().code.clone()
  }

  pub fn get_lang_view(&self, lang_id: u8) -> Data {
    let sort = self.sort.borrow();
    let mut vec= Vec::with_capacity(sort.len());
    for lang in sort.iter() {
      vec.push(lang.clone());
    }
    Data::VecLang((lang_id, vec))
  }

  // Load local translation for current controller
  pub fn lang_load(&mut self, module: &String, class: &String) {
    let i = self.i18n.borrow();
    if let Some(v) = i.get(&self.lang_id) {
      if let Some(v) = v.get(module) {
        if let Some(v) = v.get(class) {
          self.data = Some(Rc::clone(v));
        }
      }
    }
  }

  // Get a translation by key
  pub fn lang_get(&self, key: &String) -> String {
    match &self.data {
      Some(val) => {
        match val.borrow().get(key) {
          Some(v) => Action::htmlencode(v),
          None => Action::htmlencode(key),
        }
      },
      None => Action::htmlencode(key),
    }
  }

  // Replace special html text
  pub fn htmlencode(text: &String) -> String {
    let mut text = text.replace("&", "&amp;");
    text = text.replace("\"", "&quot;");
    text = text.replace("'", "&apos;");
    text = text.replace("<", "&lt;");
    text = text.replace(">", "&gt;");
    text
  }
}
