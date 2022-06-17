use std::{cell::RefCell, rc::Rc, collections::HashMap, fs::read_to_string};

use super::{response::Response, action::{Answer, Data}, lang::Lang};

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
    let file = format!("{}view_{}.html", self.dir, view);
    // Read data from template
    if let Ok(mut view) = read_to_string(&file) {
      view = view.replace("<?=lang?>", &self.response.borrow().lang);
      // Replase special marker
      for (key, data) in data {
        match data {
          Data::None => {
            let key = format!("<?={}?>", key);
            view = view.replace(&key, "");
          },
          Data::String(val) => {
            let key = format!("<?={}?>", key);
            view = view.replace(&key, val);
          },
          Data::VecLang((lang_id, val)) => {
            let key_start = format!("<?[{}?>", key);
            let key_finish = format!("<?{}]?>", key);
            if let Some(start) = view.find(&key_start) {
              if let Some(finish) = view.find(&key_finish) {
                if start<finish {
                  let tpl = view[start+key_start.len()..finish].to_string();
                  let mut vec = Vec::with_capacity(val.len());
                  for lang in val {
                    let k = format!("<?={}.lang_id?>", key);
                    let mut v = tpl.replace(&k, &lang.lang_id.to_string());
                    let k = format!("<?={}.lang?>", key);
                    v = v.replace(&k, &lang.lang);
                    let k = format!("<?={}.code?>", key);
                    v = v.replace(&k, &lang.code);
                    let k = format!("<?={}.name?>", key);
                    v = v.replace(&k, &Lang::htmlencode(&lang.name));
                    let k = format!("<?={}.selected?>", key);
                    if *lang_id == lang.lang_id {
                      v = v.replace(&k, "selected");
                    } else {
                      v = v.replace(&k, "");
                    }
                    vec.push(v);
                  }
                  view = format!("{}{}{}", &view[0..start], vec.join("") , &view[finish+key_finish.len()..])
                }
              };
            };
          },
          _ => {},
        };
      }
      return Answer::String(view);
    };

    Answer::None
  }

}