use std::{sync::{Arc, Mutex, RwLock}, cell::RefCell, rc::Rc, collections::HashMap, fs::remove_file};

use chrono::{Duration, Utc};
use postgres::Client;

use crate::app::{action::{Action, Answer}, response::Response};
use super::{worker::Worker, i18n::LangItem};

// Wrapper for the fastCGI server
pub struct Sys { }

impl Sys {
  // Constuctor
  pub fn run(
    worker: Arc<Mutex<Worker>>, 
    sql: Rc<RefCell<Client>>, 
    param: &HashMap<String, String>, 
    stdin: &Option<Vec<u8>>, 
    i18n: Rc<RefCell<HashMap<u8, 
    HashMap<String, HashMap<String, Rc<RefCell<HashMap<String, String>>>>>>>>, 
    langs: Rc<RefCell<HashMap<u8, LangItem>>>, 
    sort: Rc<RefCell<Vec<LangItem>>>,
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
    let mut action = Action::new(sql, salt, storage, i18n, param, stdin, dir, langs, sort);
    let text = match action.start() {
      // Answer::Raw(answer) => answer,
      Answer::String(answer) => answer.as_bytes().to_vec(),
      Answer::None => Vec::new(),
    };
    action.stop();

    // Prepare answer to the WEB server
    let mut answer: Vec<String> = Vec::with_capacity(16);
    answer.push("HTTP/1.1 ".to_owned());

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
      let request = action.request.borrow();
      let time = Utc::now() + Duration::seconds(cookie.time.into());
      let date: String = time.format("%a, %d-%b-%Y %H:%M:%S GMT").to_string();
      answer.push(format!("Set-Cookie: {}={}; Expires={}; Max-Age={}; path=/; domain={}; Secure; SameSite=none\r\n", cookie.key, cookie.value, date, cookie.time, request.host));
      // delete temp files
      for (_, val) in &request.file {
        for f in val {
          remove_file(&f.tmp).unwrap_or_default();
        }
      }
    }
    answer.push("Connection: keep-alive\r\n".to_owned());
    answer.push(format!("Content-Type: text/html; charset=utf-8\r\n"));
    answer.push(format!("Content-Length: {}\r\n", text.len()));
    answer.push("\r\n".to_owned());
    let mut answer = answer.join("").as_bytes().to_vec();
    answer.extend_from_slice(&text[..]);

    answer
  }
}