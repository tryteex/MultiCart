use std::{rc::Rc, cell::RefCell};

use super::{session::Session, cache::Cache, db::DB, action::Data};

// Authentification system
pub struct Auth {
  session: Rc<RefCell<Session>>,    // Session
  db: Rc<RefCell<DB>>,              // Database
  cache: Rc<RefCell<Cache>>,        // Cache
}

impl Auth {
  // Constructor
  pub fn new(session: Rc<RefCell<Session>>, db: Rc<RefCell<DB>>, cache: Rc<RefCell<Cache>>) -> Auth {
    Auth {
      session,
      db,
      cache,
    }
  }

  // Checking access to the web controller
  pub fn get_access(&mut self, module: &String, class: &String, action: &String) -> bool {
    let session = self.session.borrow();
    let mut cache = self.cache.borrow_mut();
    let mut db = self.db.borrow_mut();

    // System user always has access
    let key = "system".to_string();
    if let Some(system) = session.get(&key) {
      if let Data::Bool(v) = system {
        if *v { return true; };
      }
    }

    let key = format!("auth:{}:{}:{}:{}", session.user_id, module, class, action);
    // Check access in cache
    if let Some(a) = cache.get(&key) {
      if let Data::Bool(val) = a {
        return val.clone();
      } else {
        return false;
      }
    }

    // Prepare sql query
    let mut w: Vec<String> = Vec::with_capacity(4);
    w.push("(c.module='' AND c.class='' AND c.action='')".to_string());
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
    ", session.user_id, w.join(" OR ").to_string());
    let res = db.query(&sql);
    if res.len() == 1 {
      let row = &res[0];
      let access: i32 = row.get(0);
      if access == 1 {
        cache.set(key, Data::Bool(true));
        return true;
      }
    }
    cache.set(key, Data::Bool(false));
    false
  }
}