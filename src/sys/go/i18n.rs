use std::{collections::HashMap, fs::{read_dir, read_to_string}, io::Error};

use ini_core::{Parser, Item};

// Translation
pub struct I18n {
  pub load: bool,                                                                     // Translation is loaded
  pub langs: HashMap<u8, LangItem>,                                                   // List of enable langs
  pub sort: Vec<LangItem>,                                                            // Sorted list of langs
  pub langs_code: HashMap<String, u8>,                                                // Lang code to lang ID
  pub data: HashMap<u8, HashMap<String, HashMap<String, HashMap<String, String>>>>,   // Translations
}

// One language item
#[derive(Debug, Clone)]
pub struct LangItem {
  pub lang_id: u8,      // lang_id from database
  pub lang: String,     // Language code ISO 3166 alpha-2
  pub code: String,     // Name of native language
  pub name: String,     // Language code ISO 639-1
}

impl I18n {
  // Constructor
  pub fn new() -> I18n {
    I18n {
      load: false,
      langs: HashMap::with_capacity(8),
      sort: Vec::with_capacity(8),
      langs_code: HashMap::with_capacity(8),
      data: HashMap::with_capacity(8),
    }
  }

  // Clone language settings
  pub fn clone_lang(&self) -> (HashMap<u8, LangItem>, Vec<LangItem>) {
    let mut langs: HashMap<u8, LangItem> = HashMap::with_capacity(self.langs.len());
    let mut sort: Vec<LangItem> = Vec::with_capacity(self.sort.len());
    for (lang_id, item) in &self.langs {
      langs.insert(*lang_id, item.clone());
    }
    for item in &self.sort {
      sort.push(item.clone());
    }
    (langs, sort)
  }

  // Load translations
  pub fn load_lang(&mut self, dir: &String) -> Result<(), Error> {
    let dir = format!("{}app/", dir);
    // Read dir with application data
    match read_dir(format!("{}", dir)) {
      Ok(d1) => {
        // Get all dir in the "module" directory
        for p1 in d1 {
          match p1 {
            Ok(p1) => {
              if p1.path().is_dir() {
                match p1.file_name().to_str() {
                  Some(p1_index) => {
                    // Get all dir in the "class" directory
                    match read_dir(p1.path()) {
                      Ok(d2) => {
                        for p2 in d2 {
                          match p2 {
                            Ok(p2) => {
                              if p2.path().is_dir() {
                                match p2.file_name().to_str() {
                                  Some(p2_index) => {
                                    // Get all file with translations from the "class" directory
                                    match read_dir(p2.path()) {
                                      Ok(d3) => {
                                        for p3 in d3 {
                                          match p3 {
                                            Ok(p3) => {
                                              if p3.path().is_file() {
                                                // Decode the language
                                                match p3.file_name().to_str() {
                                                  Some(file) => {
                                                    if file.starts_with("lang_") && file.ends_with(".ini") {
                                                      let code = &file[5..file.len()-4];
                                                      if let Some(id) = self.langs_code.get(code) {
                                                        // Read translations from the file
                                                        match read_to_string(&p3.path()) {
                                                          Ok(text) => {
                                                            // Parse translations
                                                            for (size, item) in Parser::new(&text).auto_trim(true).enumerate() {
                                                              match item {
                                                                Item::Property(key, value) => {
                                                                  if !self.data.contains_key(id) {
                                                                    self.data.insert(*id, HashMap::with_capacity(256));
                                                                  }
                                                                  let data = self.data.get_mut(id).unwrap();
                                                                  if !data.contains_key(p1_index) {
                                                                    data.insert(p1_index.to_string(), HashMap::with_capacity(256));
                                                                  }
                                                                  let data = data.get_mut(p1_index).unwrap();
                                                                  if !data.contains_key(p2_index) {
                                                                    data.insert(p2_index.to_string(), HashMap::with_capacity(size));
                                                                  }
                                                                  let data = data.get_mut(p2_index).unwrap();
                                                                  data.insert(key.to_string(), value.to_string());
                                                                },
                                                                _ => {},
                                                              }
                                                            }
                                                          },
                                                          Err(e) => return Err(e),
                                                        };
                                                      }
                                                    }
                                                  },
                                                  None => {},
                                                };
                                              }
                                            },
                                            Err(e) => return Err(e),
                                          }
                                        }
                                      },
                                      Err(e) => return Err(e),
                                    };
                                  },
                                  None => {},
                                }
                              }
                            },
                            Err(e) => return Err(e),
                          };
                        }
                      },
                      Err(e) => return Err(e),
                    };
                  },
                  None => {},
                }
              }
            },
            Err(e) => return Err(e),
          };
        }
      },
      Err(e) => return Err(e),
    };
    Ok(())
  }
}