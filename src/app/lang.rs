use std::{rc::Rc, cell::RefCell, collections::HashMap};

use crate::sys::go::i18n::LangItem;

use super::{action::Data, session::Session};

// Copy data with translation
pub struct Lang {
  pub lang_id: u8,                                                                                                  // User lang_id 
  session: Rc<RefCell<Session>>,                                                                                // Session for store user selected lang_id
  i18n: Rc<RefCell<HashMap<u8, HashMap<String, HashMap<String, Rc<RefCell<HashMap<String, String>>>>>>>>,       // Global ref to tranlations
  data: Option<Rc<RefCell<HashMap<String, String>>>>,                                                           // Local copy of translation for web controller
  langs: Rc<RefCell<HashMap<u8, LangItem>>>,                                                                    // List of enable langs
  sort: Rc<RefCell<Vec<LangItem>>>,                                                                             // Sorted list of langs

}

impl Lang {
  // Constructor
  pub fn new(
    i18n: Rc<RefCell<HashMap<u8, HashMap<String, HashMap<String, Rc<RefCell<HashMap<String, String>>>>>>>>, 
    session: Rc<RefCell<Session>>, 
    langs: Rc<RefCell<HashMap<u8, LangItem>>>,
    sort: Rc<RefCell<Vec<LangItem>>>,
  ) -> Lang {
    Lang {
      lang_id: Lang::default(),
      session,
      i18n,
      data: None,
      langs,
      sort,
    }
  }

  // Get the correct default user language
  pub fn set_lang_id(&mut self, lang_id: Option<u8>) {
    let mut session = self.session.borrow_mut();

    let key = "lang_id".to_string();
    match lang_id {
      None => match session.get_lang_id() {
        Some(lang_id) => self.lang_id = lang_id,
        None => {
          let lang_id = Lang::default();
          session.set(key, Data::U8(lang_id));
          self.lang_id = lang_id;
        },
      },
      Some(lang_id) => match session.get_lang_id() {
        Some(s_lang_id) => {
          let l_id = s_lang_id;
          if l_id != lang_id {
            session.set(key, Data::U8(lang_id));
          }
          self.lang_id = lang_id
        },
        None => {
          session.set(key, Data::U8(lang_id));
          self.lang_id = lang_id
        },
      },
    }
  }

  // Get default lang_id for new user
  pub fn default() -> u8 {
    0
  }

  // Get current language code
  pub fn get_code(&self) -> String {
    self.langs.borrow().get(&self.lang_id).unwrap().code.clone()
  }

  pub fn get_lang_view(&self, lang_id: u8) -> Data {
    let sort = self.sort.borrow();
    let mut vec= Vec::with_capacity(sort.len());
    for lang in sort.iter() {
      vec.push(lang.clone());
    }
    Data::VecLang((lang_id, vec))
  }

  // Load local translation for current controller
  pub fn load(&mut self, module: &String, class: &String) {
    let i = self.i18n.borrow();
    if let Some(v) = i.get(&self.lang_id) {
      if let Some(v) = v.get(module) {
        if let Some(v) = v.get(class) {
          self.data = Some(Rc::clone(v));
        }
      }
    }
  }

  // Get a translation by key
  pub fn get(&self, key: &String) -> String {
    match &self.data {
      Some(val) => {
        match val.borrow().get(key) {
          Some(v) => Lang::htmlencode(v),
          None => Lang::htmlencode(key),
        }
      },
      None => Lang::htmlencode(key),
    }
  }

  // Replace special html text
  pub fn htmlencode(text: &String) -> String {
    let mut text = text.replace("&", "&amp;");
    text = text.replace("\"", "&quot;");
    text = text.replace("'", "&apos;");
    text = text.replace("<", "&lt;");
    text = text.replace(">", "&gt;");
    text
  }
}