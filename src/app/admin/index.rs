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
  pub fn index(&mut self, _params: &str, _data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    if self.action.get_access("admin", "index", "main") {
      self.action.set_redirect("/admin/index/main", false);
    } else {
      self.action.set_redirect("/login/admin/index", false);
    }
    
    Answer::None
  }

  //Dashboard
  pub fn main(&mut self, _params: &str, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    data.insert("company".to_owned(), Data::String(self.action.setting_get("company").unwrap()));
    self.view.out("main", data)
  }
}
