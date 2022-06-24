use std::{collections::HashMap, rc::Rc};

use crate::app::{action::{Action, Data, Answer}, view::View};

pub struct App<'a> {
  view: View, action: &'a mut Action,
}

impl<'a> App<'a> {
  pub fn new(action: &mut Action) -> App {
    let dir = format!("{}/app/{}/{}/", action.request.borrow().path, &action.module, &action.class);
    let view = View::new(Rc::clone(&action.response), dir);
    action.lang.load(&action.module, &action.class);
    App { view, action}
  }

  // Main page
  pub fn index(&mut self, _params: &str, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    data.insert("title".to_owned(), Data::String(self.action.lang.get(&"title".to_owned())));
    data.insert("enter".to_owned(), Data::String(self.action.lang.get(&"enter".to_owned())));
    data.insert("lang".to_owned(), self.action.lang.get_lang_view(self.action.lang.lang_id));
    data.insert("lang_id".to_owned(), Data::String(self.action.lang.lang_id.to_string()));
    self.view.out("login".to_owned(), data)
  }
}
