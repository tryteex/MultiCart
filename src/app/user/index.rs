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

  // Header login item
  pub fn menu(&mut self, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.set_redirect("/index/index/not_found", true);
    }
    self.view.out("menu", data)
  }
  
  // Sign up
  pub fn up(&mut self, _params: &str, _data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    if !self.action.ajax {
      self.action.set_redirect("/index/index/not_found", true);
    }
    
    Answer::None
  }
}
