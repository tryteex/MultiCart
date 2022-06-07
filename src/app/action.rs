use std::{sync::{Mutex, Arc}, rc::Rc, cell::RefCell, collections::{HashMap, BTreeMap}};

use postgres::Client;

use crate::{sys::go::{storage::Storage}};

use super::{db::DB, set::Set, cache::Cache, request::Request, response::Response, session::Session, auth::Auth, lang::Lang};

use cast::u8;

// The type of response from the controller
pub enum Answer{
  None,               // With out answer
  String(String),     // Answer in the form of text
  Raw(Vec<u8>),       // Answer in vinary data
}

// Type of data, which use in server
pub enum Data {
  None,
  I8(i8),
  U8(u8),
  I16(i16),
  U16(u16),
  I32(i32),
  U32(u32),
  I64(i64),
  U64(u64),
  I128(i128),
  U128(u128),
  ISize(isize),
  USize(usize),
  F32(f32),
  F64(f64),
  Bool(bool),
  Char(char),
  String(String),
  Vec(Vec<Data>),
  MapU8(HashMap<u8, Data>),
  MapU16(HashMap<u16, Data>),
  MapU32(HashMap<u32, Data>),
  MapU64(HashMap<u64, Data>),
  Map(HashMap<String, Data>),       // Map of string keys
  Tree(BTreeMap<String, Data>),     // Map of string keys with a clearly fixed sequence
  Raw(Vec<u8>),                     // Raw data
}

// Main CRM struct
pub struct Action {
  pub module: String,                       // Run module
  pub class: String,                        // Run class
  pub action: String,                       // Run controller
  pub salt: String,                         // Salt for password
  pub db: Rc<RefCell<DB>>,                  // Database
  pub cache: Rc<RefCell<Cache>>,            // Cache
  pub set: Rc<RefCell<Set>>,                // Setting
  pub request: Rc<RefCell<Request>>,        // Request from WEB server
  pub response: Rc<RefCell<Response>>,      // Response to WEB server
  pub session: Rc<RefCell<Session>>,        // Session
  pub auth: Rc<RefCell<Auth>>,              // Authentification system
  pub lang: Rc<RefCell<Lang>>,              // Copy data with translation
}

impl Action {
  // Constructor
  pub fn new(sql: Rc<RefCell<Client>>, salt: String, storage: Arc<Mutex<Storage>>, i18n: Rc<RefCell<HashMap<u8, HashMap<String, HashMap<String, Rc<RefCell<HashMap<String, String>>>>>>>>, param: &HashMap<String, String>, dir: String) -> Action {
    let db = Rc::new(RefCell::new(DB::new(sql)));
    let cache = Rc::new(RefCell::new(Cache::new(storage)));
    let set = Rc::new(RefCell::new(Set::new(Rc::clone(&db), Rc::clone(&cache))));
    let request = Rc::new(RefCell::new(Request::new(param, dir.clone())));
    let response = Rc::new(RefCell::new(Response::new()));
    let session = Rc::new(RefCell::new(Session::new(salt.clone(), Rc::clone(&db), Rc::clone(&request), Rc::clone(&response))));
    let auth = Rc::new(RefCell::new(Auth::new(Rc::clone(&session), Rc::clone(&db), Rc::clone(&cache))));
    let lang = Rc::new(RefCell::new(Lang::new(i18n, Rc::clone(&session))));
    let module = "".to_string();
    let class = "".to_string();
    let action = "".to_string();


    Action {
      module,
      class,
      action,
      db,
      salt,
      cache,
      set,
      request,
      response,
      session,
      auth,
      lang,
    }
  }

  // Start CRM system
  pub fn start(&mut self) -> Answer {
    // Encode routes
    if let Some((module, class, action, params, lang_id)) = self.extract_route() {
      {
        let mut lang = self.lang.borrow_mut();
        lang.set_lang_id(lang_id);
      }
      let mut data: HashMap<String, Data> = HashMap::with_capacity(256);
      // Start CRM system with fixed struct
      return self.start_route(&module, &class, &action, &params, &mut data, false);
    }
    return Answer::None;
  }

  // Encode routes
  fn extract_route(&mut self) -> Option<(String, String, String, String, Option<u8>)> {
    let request = self.request.borrow();
    let mut db = self.db.borrow_mut();
    let mut response = self.response.borrow_mut();
    let mut cache = self.cache.borrow_mut();

    // Find redirect
    let route = &request.url;
    let url = db.escape(route.to_string());
    let key = format!("redirect:{}", route);
    if let Some(data) = cache.get(&key) {
      if let Data::String(r) = data {
        let permanently = if r.starts_with("1") { true } else { false };
        response.set_redirect(r[1..].to_string(), permanently);
        return None;
      }
    }
    let sql = format!("
      SELECT redirect, permanently
      FROM redirect
      WHERE url={}
    ", url);
    let res = db.query(&sql);
    if res.len() == 1 {
      let row = &res[0];
      let redirect: String = row.get(0);
      let code: bool = row.get(1);
      let c = if code { "1" } else { "0" };
      let permanently = if code { true } else { false };
      let value = format!("{}{}", c, redirect);
      response.set_redirect(redirect, permanently);
      cache.set(key, Data::String(value));
      return None;
    }
    cache.set(key, Data::None);

    // Get route
    let key = format!("route:{}", route);
    if let Some(data) = cache.get(&key) {
      if let Data::String(r) = data {
        let res: Vec<&str> = r.splitn(5, ":").collect();
        let module = res[0].to_string();
        let class = res[1].to_string();
        let action = res[2].to_string();
        let params = res[3].to_string();
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
    let res = db.query(&sql);
    if res.len() == 1 {
      let row = &res[0];
      let module: String = row.get(0);
      let class: String = row.get(1);
      let action: String = row.get(2);
      let params: String = row.get(3);
      let lang_id: i64 = row.get(4);
      let lang_id = u8(lang_id).unwrap();
      let value = format!("{}:{}:{}:{}:{}", module, class, action, params, lang_id.to_string());
      cache.set(key, Data::String(value));
      return Some((module, class, action, params, Some(lang_id)));
    }
    cache.set(key, Data::None);

    // Encode route
    let mut module = "index".to_string();
    let mut class = "index".to_string();
    let mut action = "index".to_string();
    let mut params = "index".to_string();
    if route != "/" {
      let load: Vec<&str> = route.splitn(5, "/").collect();
      let len = load.len();
      if len == 2{
        module = load[1].to_string();
      } else if len == 3 {
        module = load[1].to_string();
        class = load[2].to_string();
      } else if len == 4 {
        module = load[1].to_string();
        class = load[2].to_string();
        action = load[3].to_string();
      } else if len == 5 {
        module = load[1].to_string();
        class = load[2].to_string();
        action = load[3].to_string();
        params = load[4].to_string();
      }
    }
    Some((module, class, action, params, None))
  }

  // Stop server
  pub fn stop(&mut self) {
    let mut session = self.session.borrow_mut();
    session.save();
  }

  // Start CRM system with fixed struct
  fn start_route(&mut self, module: &String, class: &String, action: &String, params: &String, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    // Get Access
    let access;
    {
      let mut auth = self.auth.borrow_mut();
      access = auth.get_access(module, class, action);
    }

    if access {
      // Run controller
      return self.run(module, class, action, params, data, internal);
    }
    
    // Not found
    if internal {
      return Answer::String("not_found".to_string());
    }
    {
      let mut response = self.response.borrow_mut();
      response.set_redirect("/index/index/not_found".to_string(), false);
    }
    Answer::None
  }

  // Load internal controller
  pub fn load(&mut self, module: &String, class: &String, action: &String, params: &String, data: &mut HashMap<String, Data>) -> Answer {
    self.start_route(module, class, action, params, data, true)
  }

  // Run controller
  fn run (&mut self, module: &str, class: &str, action: &str, params: &String, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    self.module = module.to_string();
    self.class = class.to_string();
    self.action = action.to_string();
    match module {
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
        _ => {},
      },
      _ => {},
    };
    Answer::None
  }

}
