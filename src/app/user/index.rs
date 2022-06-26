use std::collections::HashMap;

use crate::app::{action::{Action, Data, Answer}, view::View};

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

  // Header login item
  pub fn menu(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.set_redirect("/index/index/not_found".to_owned(), true);
    }
    self.view.out("menu".to_owned(), data)
  }
  
  // Sign up
  pub fn up(&mut self, _params: &str, _data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    if !self.action.ajax {
      self.action.set_redirect("/index/index/not_found".to_owned(), true);
    }
    
    Answer::None
  }
}
