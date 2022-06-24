use std::{collections::HashMap, rc::Rc};

use crate::app::{action::{Action, Data, Answer}, view::View};

pub struct App<'a> {
  view: View, action: &'a mut Action,
}

impl<'a> App<'a> {
  pub fn new(action: &mut Action) -> App {
    let dir = format!("{}/app/{}/{}/", action.request.borrow().path, &action.module, &action.class);
    let view = View::new(Rc::clone(&action.response), dir);
    action.lang.load(&action.module, &action.class);
    App { view, action}
  }

  // Main page
  pub fn index(&mut self, _params: &str, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    data.insert("title".to_owned(), Data::String(self.action.lang.get(&"title".to_owned())));
    if let Answer::String(a) = self.action.load("index", "index", "head", "", data) {
      data.insert("head".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = self.action.load("index", "index", "foot", "", data) {
      data.insert("foot".to_owned(), Data::String(a));
    };
    self.view.out("index".to_owned(), data)
  }
  
  // Header index
  pub fn head(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.response.borrow_mut().set_redirect("/index/index/not_found".to_owned(), true);
    }
    if let Answer::String(a) = self.action.load("index", "menu", "header", "", data) {
      data.insert("header".to_owned(), Data::String(a));
    };
    self.view.out("head".to_owned(), data)
  }

  // Footer index
  pub fn foot(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.response.borrow_mut().set_redirect("/index/index/not_found".to_owned(), true);
    }
    self.view.out("foot".to_owned(), data)
  }

  // Not found
  pub fn not_found(&mut self, _params: &str, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    if self.action.request.borrow().ajax {
      return Answer::None;
    }
    if let Answer::String(a) = self.action.load("index", "index", "head", "", data) {
      data.insert("head".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = self.action.load("index", "index", "foot", "", data) {
      data.insert("foot".to_owned(), Data::String(a));
    };
    self.action.response.borrow_mut().set_header_code(404);
    self.view.out("not_found".to_owned(), data)
  }
}
