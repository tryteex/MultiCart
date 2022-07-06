use std::collections::HashMap;

use crate::app::action::{Action, Data, Answer};

pub struct App {}

impl App {

  // Main page
  pub fn index(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    data.insert("title".to_owned(), Data::String(action.lang(&"title".to_owned())));
    data.insert("enter".to_owned(), Data::String(action.lang(&"enter".to_owned())));
    data.insert("lang".to_owned(), action.get_lang_view(action.lang_id));
    data.insert("lang_id".to_owned(), Data::String(action.lang_id.to_string()));
    action.out("login", data)
  }
}
