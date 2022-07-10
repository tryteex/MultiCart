use std::collections::HashMap;

use crate::app::action::{Action, Data, Answer};

pub struct App {}

impl App {

  // main page header search
  pub fn main(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      action.redirect_set("/index/index/not_found", true);
    }
    action.out("main", data)
  }
  
  // search in menu
  pub fn small(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      action.redirect_set("/index/index/not_found", true);
    }
    action.out("small", data)
  }
}