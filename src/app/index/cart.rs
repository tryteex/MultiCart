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

  // cart in header
  pub fn index(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.set_redirect("/index/index/not_found".to_owned(), true);
    }
    self.view.out("index".to_owned(), data)
  }
}
