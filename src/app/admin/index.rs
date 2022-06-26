use std::collections::HashMap;

use crate::app::{view::View, action::{Action, Data, Answer}};

pub struct App<'a> {
  view: View, action: &'a mut Action,
}

impl<'a> App<'a> {
  pub fn new(action: &mut Action) -> App {
    let dir = format!("{}/app/{}/{}/", action.path, &action.module, &action.class);
    let view = View::new(dir);
    action.lang_load(&action.module, &action.class);
    App { view, action}
  }

  // Main page
  pub fn index(&mut self, _params: &str, _data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    if self.action.get_access(&"admin".to_owned(), &"index".to_owned(), &"main".to_owned()) {
      self.action.set_redirect("/admin/index/main".to_owned(), false);
    } else {
      self.action.set_redirect("/login/admin/index".to_owned(), false);
    }
    
    Answer::None
  }

  //Dashboard
  pub fn main(&mut self, _params: &str, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    data.insert("company".to_owned(), Data::String(self.action.setting_get("company").unwrap()));
    self.view.out("main".to_owned(), data)
  }
}
