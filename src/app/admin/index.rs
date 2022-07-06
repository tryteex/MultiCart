use std::collections::HashMap;

use crate::app::action::{Data, Answer, Action};

pub struct App {}

impl App {

  // Main page
  pub fn index(action: &mut Action, _params: &str, _data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    if action.get_access("admin", "index", "main") {
      action.set_redirect("/admin/index/main", false);
    } else {
      action.set_redirect("/login/admin/index", false);
    }
    
    Answer::None
  }

  //Dashboard
  pub fn main(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    data.insert("company".to_owned(), Data::String(action.setting_get("company").unwrap()));
    action.out("main", data)
  }
}
