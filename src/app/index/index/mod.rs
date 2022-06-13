use std::{collections::HashMap, rc::Rc};

use crate::app::{action::{Action, Data, Answer}, view::View};

pub struct App<'a> {
  view: View, action: &'a Action,
}

impl<'a> App<'a> {
  pub fn new(action: &mut Action) -> App {
    let dir = format!("app/{}/{}/{}/", action.request.borrow().path, &action.module, &action.class);
    let view = View::new(Rc::clone(&action.response), dir);
    action.lang.borrow_mut().load(&action.module, &action.class);
    App { view, action}
  }

  // Main page
  pub fn index(&mut self, _params: &String, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    // {
    //   let lang = self.action.lang.borrow();
    //   data.insert("title".to_string(), Data::String(lang.get(&"title".to_string())));
    // }
    // match self.action.load(&"index".to_string(), &"index".to_string(), &"head".to_string(), &"".to_string(), data) {
    //   Answer::None => data.insert("head".to_string(), Data::None),
    //   Answer::String(a) => data.insert("head".to_string(), Data::String(a)),
    //   Answer::Raw(a) => data.insert("head".to_string(), Data::Raw(a)),
    // };
    // match self.action.load(&"index".to_string(), &"index".to_string(), &"foot".to_string(), &"".to_string(), data) {
    //   Answer::None => data.insert("foot".to_string(), Data::None),
    //   Answer::String(a) => data.insert("foot".to_string(), Data::String(a)),
    //   Answer::Raw(a) => data.insert("foot".to_string(), Data::Raw(a)),
    // };
    // self.view.out("index".to_string(), data)
    self.action.response.borrow_mut().set_redirect("/admin".to_string(), false);
    Answer::None
  }
  
  // Header index
  pub fn head(&mut self, _params: &String, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.response.borrow_mut().set_redirect("/index/index/not_found".to_string(), true);
    }
    self.view.out("head".to_string(), data)
  }

  // Footer index
  pub fn foot(&mut self, _params: &String, data: &mut HashMap<String, Data>, internal: bool) -> Answer {
    if !internal {
      self.action.response.borrow_mut().set_redirect("/index/index/not_found".to_string(), true);
    }
    self.view.out("foot".to_string(), data)
  }

  // Not found
  pub fn not_found(&mut self, _params: &String, data: &mut HashMap<String, Data>, _internal: bool) -> Answer {
    if self.action.request.borrow().ajax {
      return Answer::None;
    }
    self.action.response.borrow_mut().set_header_code(404);
    self.view.out("not_found".to_string(), data)
  }
}
