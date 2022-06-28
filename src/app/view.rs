use std::{collections::HashMap, fs::read_to_string};

use super::action::{Answer, Data, Action};

// Processing of html templates (Templater)
pub struct View {
  dir: String,                        // Working directory
}

impl View {
  // Constructor
  pub fn new(dir: String) -> View {
    View {
      dir,
    }
  }

  // Rendering template
  pub fn out(&self, view: &str, data: &HashMap<String, Data>) -> Answer {
    let file = format!("{}view_{}.html", self.dir, view);
    // Read data from template
    if let Ok(mut view) = read_to_string(&file) {
      view.reserve_exact(view.capacity()*2);
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
                  let tpl = view[start+key_start.len()..finish].to_owned();
                  let mut vec = Vec::with_capacity(val.len());
                  for lang in val {
                    let k = format!("<?={}.lang_id?>", key);
                    let mut v = tpl.replace(&k, &lang.lang_id.to_string());
                    let k = format!("<?={}.lang?>", key);
                    v = v.replace(&k, &lang.lang);
                    let k = format!("<?={}.code?>", key);
                    v = v.replace(&k, &lang.code);
                    let k = format!("<?={}.name?>", key);
                    v = v.replace(&k, &Action::htmlencode(&lang.name));
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