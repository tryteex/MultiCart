use std::collections::HashMap;

use crate::app::action::{Action, Data, Answer};

pub struct App {}

impl App {
  // Main page
  pub fn index(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    data.insert("lang".to_owned(), Data::String(action.lang_code.to_owned()));
    data.insert("title".to_owned(), Data::String(action.lang(&"title".to_owned())));
    if let Answer::String(a) = action.load("index", "index", "head", "", data) {
      data.insert("head".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = action.load("index", "index", "foot", "", data) {
      data.insert("foot".to_owned(), Data::String(a));
    };
    action.out("index", data)
  }
  
  // Header index
  pub fn head(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      action.set_redirect("/index/index/not_found", true);
    }
    if let Answer::String(a) = action.load("index", "menu", "header", "", data) {
      data.insert("header".to_owned(), Data::String(a));
    };
    action.out("head", data)
  }

  // Footer index
  pub fn foot(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      action.set_redirect("/index/index/not_found", true);
    }
    action.out("foot", data)
  }

  // Not found
  pub fn not_found(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    if action.ajax {
      return Answer::None;
    }
    data.insert("lang".to_owned(), Data::String(action.lang_code.to_owned()));
    if let Answer::String(a) = action.load("index", "index", "head", "", data) {
      data.insert("head".to_owned(), Data::String(a));
    };
    if let Answer::String(a) = action.load("index", "index", "foot", "", data) {
      data.insert("foot".to_owned(), Data::String(a));
    };
    action.http_code = Some(404);
    action.out("not_found", data)
  }
}
