use std::{sync::{Arc, Mutex}, collections::{HashMap, BTreeMap}};

use crate::sys::go::storage::Storage;

use super::action::Data;

// Helper to use global cache
pub struct Cache {
  storage: Arc<Mutex<Storage>>,       // Global cache
}

impl Cache {
  // Constructor
  pub fn new(storage: Arc<Mutex<Storage>>) -> Cache {
    Cache {
      storage,
    }
  }
  
  // Set value
  pub fn set(&mut self, key: String, value: Data) {
    let mut s = Mutex::lock(&self.storage).unwrap();
    s.set(key, value);
  }
  
  // Get value
  pub fn get(&self, key: &str) -> Option<Data> {
    let storage = Mutex::lock(&self.storage).unwrap();
    match storage.get(key) {
      Some(data) => {
        let val = Mutex::lock(&data).unwrap();
        Some(self.set_value(&val))
      },
      None => None,
    }
  }
  
  // Delete value
  pub fn del(&self, key: &str) {
    let s = Mutex::lock(&self.storage).unwrap();
    s.del(key);
  }

  // Clear all cache
  pub fn clear(&mut self) {
    let mut s = Mutex::lock(&self.storage).unwrap();
    s.clear();
  }

  // Checking key is set
  pub fn is_key(&self, key: &str) -> bool {
    let res;
    {
      let s = Mutex::lock(&self.storage).unwrap();
      res = s.is_key(key);
    }
    res
  }

  // Decode value
  fn set_value(&self, value: &Data) -> Data {
    match value {
      Data::None => Data::None,
      Data::I8(v) => Data::I8(v.clone()),
      Data::U8(v) => Data::U8(v.clone()),
      Data::I16(v) => Data::I16(v.clone()),
      Data::U16(v) => Data::U16(v.clone()),
      Data::I32(v) => Data::I32(v.clone()),
      Data::U32(v) => Data::U32(v.clone()),
      Data::I64(v) => Data::I64(v.clone()),
      Data::U64(v) => Data::U64(v.clone()),
      Data::I128(v) => Data::I128(v.clone()),
      Data::U128(v) => Data::U128(v.clone()),
      Data::ISize(v) => Data::ISize(v.clone()),
      Data::USize(v) => Data::USize(v.clone()),
      Data::F32(v) => Data::F32(v.clone()),
      Data::F64(v) => Data::F64(v.clone()),
      Data::Bool(v) => Data::Bool(v.clone()),
      Data::Char(v) => Data::Char(v.clone()),
      Data::String(v) => Data::String(v.clone()),
      Data::Vec(v) => {
        let mut val: Vec<Data> = Vec::with_capacity(v.len());
        for vl in v {
          val.push(self.set_value(&vl));
        }
        Data::Vec(val)
      },
      Data::MapU8(v) => {
        let mut val: HashMap<u8, Data> = HashMap::with_capacity(v.len());
        for (key, vl) in v {
          val.insert(key.clone(), self.set_value(&vl));
        }
        Data::MapU8(val)
      },
      Data::MapU16(v) => {
        let mut val: HashMap<u16, Data> = HashMap::with_capacity(v.len());
        for (key, vl) in v {
          val.insert(key.clone(), self.set_value(&vl));
        }
        Data::MapU16(val)
      },
      Data::MapU32(v) => {
        let mut val: HashMap<u32, Data> = HashMap::with_capacity(v.len());
        for (key, vl) in v {
          val.insert(key.clone(), self.set_value(&vl));
        }
        Data::MapU32(val)
      },
      Data::MapU64(v) => {
        let mut val: HashMap<u64, Data> = HashMap::with_capacity(v.len());
        for (key, vl) in v {
          val.insert(key.clone(), self.set_value(&vl));
        }
        Data::MapU64(val)
      },
      Data::Map(v) => {
        let mut val: HashMap<String, Data> = HashMap::with_capacity(v.len());
        for (key, vl) in v {
          val.insert(key.clone(), self.set_value(&vl));
        }
        Data::Map(val)
      },
      Data::Tree(v) => {
        let mut val: BTreeMap<String, Data> = BTreeMap::new();
        for (key, vl) in v {
          val.insert(key.clone(), self.set_value(&vl));
        }
        Data::Tree(val)
      },
      Data::Raw(v) => {
        let mut val: Vec<u8> = Vec::with_capacity(v.len());
        for vl in v {
          val.push(vl.clone());
        }
        Data::Raw(val)
      },
    }
  }
}
