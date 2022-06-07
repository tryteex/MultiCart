use std::{sync::{Arc, Mutex, RwLock}, cell::RefCell, rc::Rc, collections::HashMap};

use chrono::{Duration, Utc};
use postgres::Client;

use crate::app::{action::{Action, Answer}, response::Response};
use super::worker::Worker;

// Wrapper for the fastCGI server
pub struct Sys { }

impl Sys {
  // Constuctor
  pub fn run(
    worker: Arc<Mutex<Worker>>, 
    sql: Rc<RefCell<Client>>, 
    param: &HashMap<String, String>, 
    i18n: Rc<RefCell<HashMap<u8, 
    HashMap<String, HashMap<String, Rc<RefCell<HashMap<String, String>>>>>>>>
  ) -> Vec<u8> {

    let storage;
    let salt;
    let dir;
    // Coonect the memory cache system
    {
      let w = Mutex::lock(&worker).unwrap();
      let g = Mutex::lock(&w.go).unwrap();
      storage = Arc::clone(&g.storage);
      let i = RwLock::read(&g.init).unwrap();
      salt = i.salt.clone();
      dir = i.dir.clone();
    }
    // Run CRM
    let mut action = Action::new(sql, salt, storage, i18n, param, dir);
    let text = match action.start() {
      Answer::Raw(answer) => answer,
      Answer::String(answer) => answer.as_bytes().to_vec(),
      Answer::None => Vec::new(),
    };
    action.stop();

    // Prepare answer to the WEB server
    let mut answer: Vec<String> = Vec::with_capacity(16);
    answer.push("HTTP/1.1 ".to_string());

    let response = action.response.borrow();
    if let Some(location) = response.get_redirect() {
      if location.permanently {
        answer.push(format!("{}\r\n", Response::get_code(301)));
      } else {
        answer.push(format!("{}\r\n", Response::get_code(302)));
      }
      answer.push(format!("{}\r\n", location.url));
    } else if let Some(code) = response.get_header_code() {
      answer.push(format!("{}\r\n", Response::get_code(*code)));
    } else {
      answer.push(format!("{}\r\n", Response::get_code(200)));
    }
    if let Some(cookie) = response.get_cookie() {
      let time = Utc::now() + Duration::seconds(cookie.time.into());
      let date: String = time.format("%a, %d-%b-%Y %H:%M:%S GMT").to_string();
      answer.push(format!("Set-Cookie: {}={}; Expires={}; Max-Age={}; path=/\r\n", cookie.key, cookie.value, date, cookie.time));
    }
    answer.push("Connection: keep-alive\r\n".to_string());
    answer.push(format!("Content-Length: {}\r\n", text.len()));
    answer.push("\r\n".to_string());
    let mut answer = answer.join("").as_bytes().to_vec();
    answer.extend_from_slice(&text[..]);

    answer
  }
}