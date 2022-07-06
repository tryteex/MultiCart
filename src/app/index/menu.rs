use std::collections::HashMap;

use crate::app::action::{Action, Data, Answer};

pub struct App {}

impl App{

  // Main header
  pub fn header(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      action.set_redirect("/index/index/not_found", true);
    }
    if let Answer::String(a) = action.load("index", "menu", "upper", "", data) {
      data.insert("upper".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = action.load("index", "menu", "logo", "", data) {
      data.insert("logo".to_owned(), Data::String(a));
    };
    action.out("header", data)
  }
  
  // Products main menu
  pub fn products(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      action.set_redirect("/index/index/not_found", true);
    }
    action.out("products", data)
  }
  
  // Main menu
  pub fn list(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      action.set_redirect("/index/index/not_found", true);
    }
    action.out("list", data)
  }
  
  // Logo + Search + user + cart
  pub fn logo(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      action.set_redirect("/index/index/not_found", true);
    }
    if let Answer::String(a) = action.load("index", "cart", "index", "", data) {
      data.insert("cart".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = action.load("index", "menu", "products", "", data) {
      data.insert("products".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = action.load("index", "search", "main", "", data) {
      data.insert("search".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = action.load("index", "search", "small", "", data) {
      data.insert("subsearch".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = action.load("index", "menu", "list", "", data) {
      data.insert("list".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = action.load("user", "index", "menu", "", data) {
      data.insert("user".to_owned(), Data::String(a));
    };
    action.out("logo", data)
  }
  
  // Upper menu
  pub fn upper(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      action.set_redirect("/index/index/not_found", true);
    }
    action.out("upper", data)
  }
}
