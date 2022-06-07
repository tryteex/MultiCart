use std::{cell::RefCell, rc::Rc, collections::HashMap, fs::read_to_string};

use super::{response::Response, action::{Answer, Data}};

// Processing of html templates (Templater)
pub struct View {
  response: Rc<RefCell<Response>>,    // Response data
  dir: String,                        // Working directory
}

impl View {
  // Constructor
  pub fn new(response: Rc<RefCell<Response>>, dir: String) -> View {
    View {
      response,
      dir,
    }
  }

  // Rendering template
  pub fn out(&self, view: String, data: &HashMap<String, Data>) -> Answer {
    let file = format!("{}view_{}.tpl", self.dir, view);
    // Read data from template
    if let Ok(mut view) = read_to_string(&file) {
      for (key, val) in data {
        if let Data::String(val) = val {
          // Replase special marker
          let key = format!("<?{}?>", key);
          view = view.replace(&key, val);
          // Replase cycles
          // ...
          // Special function
          // ...
        };
      }
      return Answer::String(view);
    };

    Answer::None
  }

}