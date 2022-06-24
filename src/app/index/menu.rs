use std::collections::HashMap;

use crate::app::{action::{Action, Data, Answer}, view::View};

pub struct App<'a> {
  view: View, action: &'a mut Action,
}

impl<'a> App<'a> {
  pub fn new(action: &mut Action) -> App {
    let dir = format!("{}/app/{}/{}/", action.request.borrow().path, &action.module, &action.class);
    let view = View::new(dir);
    action.lang.load(&action.module, &action.class);
    App { view, action}
  }

  // Main header
  pub fn header(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.response.set_redirect("/index/index/not_found".to_owned(), true);
    }
    if let Answer::String(a) = self.action.load("index", "menu", "upper", "", data) {
      data.insert("upper".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = self.action.load("index", "menu", "logo", "", data) {
      data.insert("logo".to_owned(), Data::String(a));
    };
    self.view.out("header".to_owned(), data)
  }
  
  // Products main menu
  pub fn products(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.response.set_redirect("/index/index/not_found".to_owned(), true);
    }
    self.view.out("products".to_owned(), data)
  }
  
  // Main menu
  pub fn list(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.response.set_redirect("/index/index/not_found".to_owned(), true);
    }
    self.view.out("list".to_owned(), data)
  }
  
  // Logo + Search + user + cart
  pub fn logo(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.response.set_redirect("/index/index/not_found".to_owned(), true);
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
    self.view.out("logo".to_owned(), data)
  }
  
  // Upper menu
  pub fn upper(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.response.set_redirect("/index/index/not_found".to_owned(), true);
    }
    self.view.out("upper".to_owned(), data)
  }
}
