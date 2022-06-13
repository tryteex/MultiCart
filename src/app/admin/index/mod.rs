use std::{collections::HashMap, rc::Rc};

use crate::app::{action::{Action, Data, Answer}, view::View};

pub struct App<'a> {
  view: View, action: &'a mut Action,
}

impl<'a> App<'a> {
  pub fn new(action: &mut Action) -> App {
    let dir = format!("app/{}/{}/{}/", action.request.borrow().path, &action.module, &action.class);
    let view = View::new(Rc::clone(&action.response), dir);
    action.lang.borrow_mut().load(&action.module, &action.class);
    App { view, action}
  }

  // Main page
  pub fn index(&mut self, _params: &String, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    if self.action.auth.borrow_mut().get_access(&"admin".to_string(), &"index".to_string(), &"main".to_string()) {
      self.action.response.borrow_mut().set_redirect("/admin/index/main".to_string(), false);
    } else {
      self.action.response.borrow_mut().set_redirect("/login/admin/index".to_string(), false);
    }

    
    Answer::None
  }
}
