use std::{collections::HashMap, rc::Rc};

use crate::app::{action::{Action, Data, Answer}, view::View};

pub struct App<'a> {
  view: View, action: &'a Action,
}

impl<'a> App<'a> {
  pub fn new(action: &mut Action) -> App {
    let dir = format!("{}/app/{}/{}/", action.request.borrow().path, &action.module, &action.class);
    let view = View::new(Rc::clone(&action.response), dir);
    action.lang.borrow_mut().load(&action.module, &action.class);
    App { view, action}
  }

  // Main page
  pub fn index(&mut self, _params: &String, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    let lang = self.action.lang.borrow();
    data.insert("title".to_string(), Data::String(lang.get(&"title".to_string())));
    data.insert("enter".to_string(), Data::String(lang.get(&"enter".to_string())));
    data.insert("lang".to_string(), lang.get_lang_view(lang.lang_id));
    data.insert("lang_id".to_string(), Data::String(lang.lang_id.to_string()));
    self.view.out("login".to_string(), data)
  }
}
