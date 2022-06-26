use std::{rc::Rc, cell::RefCell};

use postgres::{Client, Row};

use cast::u64;
use postgres_protocol::escape::escape_literal;

// Database
pub struct DB {
  sql: Rc<RefCell<Client>>,     // Postgresql connection
  pub counts: u64,              // Count selected data
  pub err: bool,                // Error of sql query
  pub query: String,            // Last sql text
  pub error: String,            // Error text
}

impl DB {
  // Constructor
  pub fn new(sql: Rc<RefCell<Client>>) -> DB {
    DB {
      sql,
      counts: 0,
      err: false,
      query: String::with_capacity(65536),
      error: String::with_capacity(65536),
    }
  }

  // Execute query to database
  pub fn query(&mut self, sql: &String)->Vec<Row> {
    let mut db = self.sql.borrow_mut();
    
    match db.query(sql, &[]) {
      Ok(res) => {
        self.err = false;
        self.counts = u64(res.len());
        self.error = "".to_owned();
        self.query = sql.to_owned();
        return res;
      },
      Err(e) => {
        self.err = true;
        self.counts = 0;
        self.error = e.to_string();
        self.query = sql.to_owned();
        return Vec::new();
      },
    };
  }

  // Escape text.
  pub fn escape(&self, text: &str)->String {
    escape_literal(&text)
  }
}