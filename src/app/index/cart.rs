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

  // cart in header
  pub fn index(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.set_redirect("/index/index/not_found", true);
    }
    self.view.out("index", data)
  }
}
