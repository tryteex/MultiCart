use std::collections::HashMap;

use crate::app::action::{Action, Data, Answer};

pub struct App {}

impl App {

  // cart in header
  pub fn index(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      action.set_redirect("/index/index/not_found", true);
    }
    action.out("index", data)
  }
}
