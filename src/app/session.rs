use std::{collections::HashMap, cell::RefCell, rc::Rc};

use chrono::Local;
use regex::Regex;
use serde_json::{Value, Map, Number};
use sha3::{Digest, Sha3_512};

use super::{db::DB, request::Request, response::Response, action::Data};

use cast::u8;

pub const ON_YEAR: u32 = 31622400;

// Manage user sessions
pub struct Session {
  pub user_id: i64,                 // user_id from database
  pub session_id: i64,              // session_id from database
  pub session: String,              // cookie key
  data: HashMap<String, Data>,      // User data
  change: bool,                     // User data is changed
  db: Rc<RefCell<DB>>,              // Database
  request: Rc<RefCell<Request>>,    // HTTP request
}

impl Session {
  // Constructor
  pub fn new(salt: String, db: Rc<RefCell<DB>>, request: Rc<RefCell<Request>>, response: Rc<RefCell<Response>>) -> Session {
    let mut response_load = response.borrow_mut();
    let mut session;
    let mut change = false;
    let data = HashMap::new();
    let key = "tryteex".to_string();
    {
      // Get and check cookie
      let request_load = request.borrow();
      session = match request_load.cookie.get(&key) {
        Some(s) => {
          match Regex::new(r"^[a-f0-9]{128}$") {
            Ok(rx) => {
              if rx.is_match(s) {
                s.to_owned()
              } else {
                "".to_string()
              }
            },
            Err(_) => "".to_string(),
          }
        },
        None => "".to_string(),
      };
      if session.len() == 0 {
        // Generate a new cookie
        let time = Local::now().format("%Y.%m.%d %H:%M:%S%.9f %:z").to_string();
        let cook = format!("{}{}{}{}{}", salt, request_load.ip, request_load.agent, request_load.host, time);
        let mut hasher = Sha3_512::new();
        hasher.update(cook.as_bytes());
        session = format!("{:#x}", hasher.finalize());
        change = true;
      }
    }
    
    // set cookie
    response_load.set_cookie(key, session.clone(), ON_YEAR);

    let mut session = Session {
      user_id: 0,
      session_id: 0,
      session,
      data,
      change,
      db,
      request,
    };
    // Load the user data
    session.load();
    session
  }

  // Get user lang_id
  pub fn get_lang_id(&self) -> Option<u8> {
    let key = "lang_id".to_string();
    match self.get(&key) {
        Some(d) => match d {
          Data::U8(v) => Some(*v),
          Data::I64(v) => Some(u8(*v).unwrap()),
          _ => None
        },
        None => None,
    }
  }

  // Set a session data
  pub fn set(&mut self, key: String, val: Data) {
    self.data.insert(key, val);
    self.change = true;
  }

  // Get a session date
  pub fn get(&self, key: &String) -> Option<&Data> {
    self.data.get(key)
  }

  // Delete a session date
  // pub fn del(&mut self, key: &String) {
  //   self.change = true;
  //   self.data.remove(key);
  // }

  // Is set a session date
  // pub fn is_key(&mut self, key: &String) -> bool {
  //   if key == "user_id" {
  //     return true;
  //   }
  //   return self.data.contains_key(key);
  // }

  // Clear the all session date
  // pub fn clear(&mut self) {
  //   if self.data.len() > 0 || self.user_id > 0 {
  //     self.data.clear();
  //     self.change = true;
  //   }
  // }

  // Load user session date from database 
  fn load(&mut self) {
    let mut db = self.db.borrow_mut();
    let request = self.request.borrow();
    let ses_esc = db.escape(self.session.clone());
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
    ", ses_esc, db.escape(request.ip.clone()), db.escape(request.agent.clone()), ses_esc);
    let res = db.query(&sql);
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
        self.data.insert(key, self.get_value(val));
      }
    };
  }

  // Decode user date from json
  fn get_value(&self, val: Value) -> Data {
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
          vec.push(self.get_value(vl));
        }
        Data::Vec(vec)
      },
      Value::Object(v) => {
        let mut map: HashMap<String, Data> = HashMap::with_capacity(v.len());
        for (k, vl) in v {
          map.insert(k, self.get_value(vl));
        }
        Data::Map(map)
      },
    }
  }

  // Encode user data to json
  fn set_value(&self, val: &Data) -> Value {
    match val {
      Data::None 
      // | Data::I128(_) | Data::U128(_) | Data::Raw(_) 
      | Data::VecLang(_) => Value::Null,
      // Data::I8(v) => Value::Number(Number::from(v.clone())),
      Data::U8(v) => Value::Number(Number::from(v.clone())),
      // Data::I16(v) => Value::Number(Number::from(v.clone())),
      // Data::U16(v) => Value::Number(Number::from(v.clone())),
      // Data::I32(v) => Value::Number(Number::from(v.clone())),
      // Data::U32(v) => Value::Number(Number::from(v.clone())),
      Data::I64(v) => Value::Number(Number::from(v.clone())),
      Data::U64(v) => Value::Number(Number::from(v.clone())),
      // Data::ISize(v) => Value::Number(Number::from(v.clone())),
      // Data::USize(v) => Value::Number(Number::from(v.clone())),
      // Data::F32(v) => Value::Number(Number::from_f64(v.clone().into()).unwrap()),
      Data::F64(v) => Value::Number(Number::from_f64(v.clone()).unwrap()),
      Data::Bool(v) => Value::Bool(v.clone()),
      // Data::Char(v) => Value::String(v.to_string()),
      Data::String(v) => Value::String(v.clone()),
      Data::Vec(v) => {
        let mut val: Vec<Value> = Vec::with_capacity(v.len());
        for vl in v {
          val.push(self.set_value(vl));
        }
        Value::Array(val)
      },
      // Data::MapU8(v) => {
      //   let mut val: Map<String, Value> = Map::with_capacity(v.len());
      //   for (key, vl) in v {
      //     val.insert(key.to_string(), self.set_value(vl));
      //   }
      //   Value::Object(val)
      // },
      // Data::MapU16(v) => {
      //   let mut val: Map<String, Value> = Map::with_capacity(v.len());
      //   for (key, vl) in v {
      //     val.insert(key.to_string(), self.set_value(vl));
      //   }
      //   Value::Object(val)
      // },
      // Data::MapU32(v) => {
      //   let mut val: Map<String, Value> = Map::with_capacity(v.len());
      //   for (key, vl) in v {
      //     val.insert(key.to_string(), self.set_value(vl));
      //   }
      //   Value::Object(val)
      // },
      // Data::MapU64(v) => {
      //   let mut val: Map<String, Value> = Map::with_capacity(v.len());
      //   for (key, vl) in v {
      //     val.insert(key.to_string(), self.set_value(vl));
      //   }
      //   Value::Object(val)
      // },
      Data::Map(v) => {
        let mut val: Map<String, Value> = Map::with_capacity(v.len());
        for (key, vl) in v {
          val.insert(key.to_string(), self.set_value(vl));
        }
        Value::Object(val)
      },
      // Data::Tree(v) => {
      //   let mut val: Map<String, Value> = Map::with_capacity(v.len());
      //   for (key, vl) in v {
      //     val.insert(key.to_string(), self.set_value(vl));
      //   }
      //   Value::Object(val)
      // },
    }
  }

  // Save session date to database
  pub fn save(&mut self) {
    let mut db = self.db.borrow_mut();
    let request = self.request.borrow();
    if self.change {
      // if data were chaged then save it
      let mut map = Map::with_capacity(self.data.len());
      for (key, val) in &self.data {
        map.insert(key.clone(), self.set_value(val));
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
      ", self.user_id, data, db.escape(request.ip.clone()), db.escape(request.agent.clone()), self.session_id);
      db.query(&sql);
    } else {
      // If date not changed, only update last visit time
      let sql = format!("
        UPDATE session 
        SET 
          last = now()
        WHERE
          session_id={}
      ", self.session_id);
      db.query(&sql);
    }
  }
}
