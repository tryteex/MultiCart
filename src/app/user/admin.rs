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

  // Main page
  pub fn index(&mut self, _params: &str, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    data.insert("title".to_owned(), Data::String(self.action.lang_get(&"title".to_owned())));
    data.insert("enter".to_owned(), Data::String(self.action.lang_get(&"enter".to_owned())));
    data.insert("lang".to_owned(), self.action.get_lang_view(self.action.lang_id));
    data.insert("lang_id".to_owned(), Data::String(self.action.lang_id.to_string()));
    self.view.out("login", data)
  }
}
