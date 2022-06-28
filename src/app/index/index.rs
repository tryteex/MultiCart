use std::collections::HashMap;

use crate::app::{view::View, action::{Action, Data, Answer}};

pub struct App<'a> {
  view: View, action: &'a mut Action,
}

impl App<'_> {
  pub fn new<'a>(action: &'a mut Action, module: &'a String, class: &'a String) -> App<'a> {
    let dir = format!("{}/app/{}/{}/", action.path, module, class);
    let view = View::new(dir);
    action.lang_load(module, class);
    App { view, action}
  }

  // Main page
  pub fn index(&mut self, _params: &str, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    data.insert("lang".to_owned(), Data::String(self.action.lang_code.to_owned()));
    data.insert("title".to_owned(), Data::String(self.action.lang_get(&"title".to_owned())));
    if let Answer::String(a) = self.action.load("index", "index", "head", "", data) {
      data.insert("head".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = self.action.load("index", "index", "foot", "", data) {
      data.insert("foot".to_owned(), Data::String(a));
    };
    self.view.out("index", data)
  }
  
  // Header index
  pub fn head(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.set_redirect("/index/index/not_found", true);
    }
    if let Answer::String(a) = self.action.load("index", "menu", "header", "", data) {
      data.insert("header".to_owned(), Data::String(a));
    };
    self.view.out("head", data)
  }

  // Footer index
  pub fn foot(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.set_redirect("/index/index/not_found", true);
    }
    self.view.out("foot", data)
  }

  // Not found
  pub fn not_found(&mut self, _params: &str, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    if self.action.ajax {
      return Answer::None;
    }
    data.insert("lang".to_owned(), Data::String(self.action.lang_code.to_owned()));
    if let Answer::String(a) = self.action.load("index", "index", "head", "", data) {
      data.insert("head".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = self.action.load("index", "index", "foot", "", data) {
      data.insert("foot".to_owned(), Data::String(a));
    };
    self.action.http_code = Some(404);
    self.view.out("not_found", data)
  }
}
