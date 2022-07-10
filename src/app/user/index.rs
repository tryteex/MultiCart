use std::collections::HashMap;

use crate::app::action::{Action, Data, Answer};

pub struct App {}

impl App {

  // Header login item
  pub fn menu(action: &mut Action, _params: &str, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      action.redirect_set("/index/index/not_found", true);
    }
    action.out("menu", data)
  }
  
  // Sign up
  pub fn up(action: &mut Action, _params: &str, _data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    if !action.ajax {
      action.redirect_set("/index/index/not_found", true);
    }
    Answer::None
  }
}
