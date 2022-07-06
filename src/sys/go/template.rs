use std::{collections::HashMap, io::Error, fs::{read_dir, read_to_string}};

// Templates system
pub struct Template {
  pub load: bool,                                                   // Template loaded
  pub tpls: HashMap<String, HashMap<String, HashMap<String, String>>>,  // List of templates
}

impl Template {
  pub fn new() -> Template {
    Template {
      load: false,
      tpls: HashMap::with_capacity(32),
    }
  }

  // Load templates
  pub fn load_templates(&mut self, dir: &str) -> Result<(), Error> {
    // Read dir with application data
    match read_dir(dir) {
      Ok(d1) => {
        // Get all dir in the "module" directory
        for p1 in d1 {
          match p1 {
            Ok(p1) => {
              if p1.path().is_dir() {
                match p1.file_name().to_str() {
                  Some(p1_index) => {
                    if !self.tpls.contains_key(p1_index) {
                      self.tpls.insert(p1_index.to_owned(), HashMap::with_capacity(32));
                    }
                    let m =  self.tpls.get_mut(p1_index).unwrap();
                    // Get all dir in the "class" directory
                    match read_dir(p1.path()) {
                      Ok(d2) => {
                        for p2 in d2 {
                          match p2 {
                            Ok(p2) => {
                              if p2.path().is_dir() {
                                match p2.file_name().to_str() {
                                  Some(p2_index) => {
                                    if !m.contains_key(p2_index) {
                                      m.insert(p2_index.to_owned(), HashMap::with_capacity(32));
                                    }
                                    let c = m.get_mut(p2_index).unwrap();
                                    // Get all file from the "class" directory
                                    match read_dir(p2.path()) {
                                      Ok(d3) => {
                                        for p3 in d3 {
                                          match p3 {
                                            Ok(p3) => {
                                              if p3.path().is_file() {
                                                // Decode the language
                                                match p3.file_name().to_str() {
                                                  Some(file) => {
                                                    if file.starts_with("view_") && file.ends_with(".html") {
                                                      let view = &file[5..file.len()-5];
                                                      let file = format!("{}{}/{}/{}", dir, p1_index, p2_index, file);
                                                      match read_to_string(&file) {
                                                        Ok(t) => {
                                                          c.insert(view.to_owned(), t);
                                                        },
                                                        Err(e) => return Err(e),
                                                      }
                                                    }
                                                  },
                                                  None => {},
                                                }
                                              }
                                            },
                                            Err(e) => return Err(e),
                                          }
                                        }
                                      },
                                      Err(e) => return Err(e),
                                    }
                                  },
                                  None => {},
                                }
                              }
                            },
                            Err(e) => return Err(e),
                          }
                        }
                      },
                      Err(e) => return Err(e),
                    }
                  },
                  None => {},
                }
              }
            },
            Err(e) => return Err(e),
          }
        }
      },
      Err(e) => return Err(e),
    };
    Ok(())
  }
}