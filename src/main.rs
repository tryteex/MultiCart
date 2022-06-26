mod sys {
  pub mod app;
  pub mod go {
    pub mod go;
    pub mod worker;
    pub mod fastcgi;
    pub mod sys;
    pub mod storage;
    pub mod i18n;
  }
  pub mod log;
  pub mod help;
  pub mod init;
}
mod app {
  pub mod admin {
    pub mod index;
  }
  pub mod index {
    pub mod index;
    pub mod menu;
    pub mod search;
    pub mod cart;
  }
  pub mod user {
    pub mod admin;
    pub mod index;
  }
  pub mod action;
  pub mod view;
}

use std::{env, sync::{Arc, RwLock}};

use sys::{init::{Init, AppAction}, go::go::Go, log::LogApp, app::App, help::Help};

// Program entry point
fn main() {
  // Initializing the logging system
  let mut log = LogApp::new();
  // Initializing the init system 
  let mut init = Init::new().unwrap_or_else(|e| { log.exit_err(&e) });
  
  log.set(init.id, init.dir.clone());

  // Loading the configuration file
  if let Err(e) = init.load() { log.exit_err(&e);};

  // Reading program parameters
  if let Err(e) = init.args(&mut env::args()) { log.exit_err(&e); };
  match init.app {
    // Start the server in the background stream and exit
    AppAction::Start => App::start(&init, &log),
    // Start FastCGI and CRM server
    AppAction::Go => {
      let i = Arc::new(RwLock::new(init));
      let l = Arc::new(RwLock::new(log));
      Go::start(i, l);
    },
    // Send an IRC "stop" signal and exit
    AppAction::Stop => App::stop(&init, &log),
    // Show help
    AppAction::Help => Help::help(),
  }
}

