use std::{cell::RefCell, rc::Rc};

use super::{db::DB, cache::Cache, action::Data};

// Get setting from database
pub struct Set{
  db: Rc<RefCell<DB>>,          // Database
  cache: Rc<RefCell<Cache>>,    // Cache
}

impl Set {
  // Constructor
  pub fn new(db: Rc<RefCell<DB>>, cache: Rc<RefCell<Cache>>) -> Set {
    Set {
      db,
      cache,
    }
  }
  
  // Getting setting
  pub fn get(&mut self, key: &String) -> Option<String> {
    let mut db = self.db.borrow_mut();
    let mut cache = self.cache.borrow_mut();

    let cache_key = format!("setting:{}", key);
    // Check cache
    if let Some(data) = cache.get(&cache_key) {
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
      ", db.escape(key.to_owned()));
    let res = db.query(&sql);
    if res.len() == 0 {
      return None;
    }
    let row = &res[0];
    let value: String = row.get(0);
    cache.set(cache_key, Data::String(value.clone()));
    Some(value)
  }
}