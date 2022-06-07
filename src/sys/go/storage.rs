use std::{collections::HashMap, sync::{Mutex, Arc}};

use crate::app::action::Data;

// The memory cache system
pub struct Storage {
  data: HashMap<String, Data>,
}

impl Storage {
  // Constructor
  pub fn new() -> Storage {
    Storage {
      data: HashMap::with_capacity(2048),
    }
  }
  
  // Set the value
  pub fn set(&mut self, key: String, value: Data) {
    self.data.insert(key, value);
  }
  
  // Get the value
  pub fn get(&self, key: &str) -> Option<Arc<Mutex<&Data>>> {
    match self.data.get(key) {
      Some(data) => Some(Arc::new(Mutex::new(data))),
      None => None,
    }
  }
  
  // Delete the value
  pub fn del(&self, key: &str) {
    for str in self.data.keys() {
      if key.starts_with(key) {
        self.del(str);
      }
    }
  }

  // Checking key is set
  pub fn is_key(&self, key: &str) -> bool {
    self.data.contains_key(key)
  }
  
  // Clear all value
  pub fn clear(&mut self) {
    self.data.clear();
  }
}
