use std::collections::HashMap;

use crate::app::{action::{Action, Data, Answer}, view::View};

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

  // Main header
  pub fn header(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.set_redirect("/index/index/not_found", true);
    }
    if let Answer::String(a) = self.action.load("index", "menu", "upper", "", data) {
      data.insert("upper".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = self.action.load("index", "menu", "logo", "", data) {
      data.insert("logo".to_owned(), Data::String(a));
    };
    self.view.out("header", data)
  }
  
  // Products main menu
  pub fn products(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.set_redirect("/index/index/not_found", true);
    }
    self.view.out("products", data)
  }
  
  // Main menu
  pub fn list(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.set_redirect("/index/index/not_found", true);
    }
    self.view.out("list", data)
  }
  
  // Logo + Search + user + cart
  pub fn logo(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.set_redirect("/index/index/not_found", true);
    }
    if let Answer::String(a) = self.action.load("index", "cart", "index", "", data) {
      data.insert("cart".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = self.action.load("index", "menu", "products", "", data) {
      data.insert("products".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = self.action.load("index", "search", "main", "", data) {
      data.insert("search".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = self.action.load("index", "search", "small", "", data) {
      data.insert("subsearch".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = self.action.load("index", "menu", "list", "", data) {
      data.insert("list".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = self.action.load("user", "index", "menu", "", data) {
      data.insert("user".to_owned(), Data::String(a));
    };
    self.view.out("logo", data)
  }
  
  // Upper menu
  pub fn upper(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.set_redirect("/index/index/not_found", true);
    }
    self.view.out("upper", data)
  }
}
