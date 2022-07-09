use std::{process, net::SocketAddr, env::{self, Args}, str::FromStr, fs::read_to_string};

use ini_core::{Parser, Item};

use super::log::LogApp;

// Database connection
pub struct DB {
  pub host: String,                // Database server
  pub port: String,                // Database port
  pub user: String,                // Database user
  pub pwd: String,                 // Database user password
  pub name: String,                // Database name
}

// Process management
pub struct Sys {
  pub max_connection: u16,            // Maximum number of connections
  pub socket: Vec<SocketAddr>,        // List of sockets to listen to
  pub irc: SocketAddr,                // IRC socket for server management
}

// Program action
pub enum AppAction {
  Start,                          // Start the server in the background stream
  Go,                             // Start the server
  Stop,                           // Stop the server
  Help,                           // Display help information
}

// Initial startup structure
pub struct Init {
  pub id: u32,                        // Operating system process ID
  pub exe: String,                    // Current program path
  pub dir: String,                    // Current startup directory
  pub sys: Sys,                       // Process management
  pub version: String,                // Version
  pub db: DB,                         // Database connection
  pub app: AppAction,                 // Program action
  pub time_zone: String,              // Timezone for database
  pub salt: String,                   // Salt for password
}

impl Init {

  // Constructor
  pub fn new() -> Result<Init, String> {
    let dir = env::current_dir().unwrap().to_str().unwrap().to_owned();
    let exe = env::current_exe().unwrap().to_str().unwrap().to_owned();
    let exe = exe.split(&dir).last().unwrap();
    let exe = format!("{}{}", dir, exe);

    let sys = Sys {
      max_connection: 25,
      socket: vec![SocketAddr::from_str("127.0.0.1:9001").unwrap()],
      irc: SocketAddr::from_str("127.0.0.1:9001").unwrap(),
    };

    let db = DB { 
      host: String::from("127.0.0.1"), 
      port: String::from("5432"), 
      user: String::from("user"), 
      pwd: String::from("pwd"), 
      name: String::from("name"),
    };
    
    Ok(Init { 
      id: process::id(),
      exe,
      dir,
      sys,
      version: env!("CARGO_PKG_VERSION").to_owned(),
      db,
      app: AppAction::Help,
      time_zone: "".to_owned(),
      salt: "".to_owned(),
    })
  }

  // Loading the configuration file
  pub fn load(&mut self) -> Result<(), String> {
    // Get config file name
    let file_name = format!("{}/tryteex.conf", self.dir);
    // Read data
    let conf = match read_to_string(&file_name) {
      Ok(conf) => conf,
      Err(err) => return Err(LogApp::get_error(100, &err.to_string())),
    };
    for item in Parser::new(&conf).auto_trim(true).enumerate() {
      match item.1 {
        Item::Property(key, value) => match key {
          "max_connection" => match value.parse::<u16>() {
            Ok(val) => match val {
              0 => return Err(LogApp::get_error(101, value)),
              _ => self.sys.max_connection = val,
            },
            Err(_) => return Err(LogApp::get_error(102, value)),
          },
          "socket" => {
            let mut val: Vec<SocketAddr> = Vec::new();
            for v in value.split(",") {
              match SocketAddr::from_str(v) {
                Ok(s) => val.push(s),
                Err(_) => return Err(LogApp::get_error(104, v)),
              }
            }
            if self.sys.socket.len() == 0 {
              return Err(LogApp::get_error(103, ""));
            }
            self.sys.socket = val;
          },
          "irc" => match value.parse::<u16>() {
            Ok(val) => match val {
              0 =>return Err(LogApp::get_error(105, value)),
              _ => self.sys.irc = SocketAddr::from_str(&format!("127.0.0.1:{}", value)).unwrap(),
            },
            Err(_) => return Err(LogApp::get_error(106, value)),
          },
          "dir" => {
            match value.trim().len() {
              0 => return Err(LogApp::get_error(107, value)),
              1024.. => return Err(LogApp::get_error(108, value)),
              _ => self.dir = value.to_owned(),
            }
          },
          "version" => {
            match value.trim().len() {
              0 => return Err(LogApp::get_error(109, value)),
              12.. => return Err(LogApp::get_error(110, value)),
              _ => self.version = value.to_owned(),
            }
          },
          "db_host" => {
            match value.trim().len() {
              0 => return Err(LogApp::get_error(111, value)),
              _ => self.db.host = value.to_owned(),
            }
          },
          "db_port" => {
            match value.trim().len() {
              0 => return Err(LogApp::get_error(112, value)),
              _ => self.db.port = value.to_owned(),
            }
          },
          "db_user" => {
            match value.trim().len() {
              0 => return Err(LogApp::get_error(113, value)),
              _ => self.db.user = value.to_owned(),
            }
          },
          "db_pwd" => {
            match value.trim().len() {
              0 => return Err(LogApp::get_error(114, value)),
              _ => self.db.pwd = value.to_owned(),
            }
          },
          "db_name" => {
            match value.trim().len() {
              0 => return Err(LogApp::get_error(115, value)),
              _ => self.db.name = value.to_owned(),
            }
          },
          "time_zone" => self.time_zone = value.trim().to_owned(),
          "salt" => self.salt = value.trim().to_owned(),
         _ => {},
        },
        _ => {},
      }
    }
    if self.salt.len() == 0 {
      return Err(LogApp::get_error(116, ""));
    }
    Ok(())
  }

  // Reading program parameters
  pub fn args(&mut self, args: &mut Args) -> Result<(), String> {
    args.next();
    self.app = match args.next() {
      None => return Ok(()),
      Some(arg) => match arg.as_str() {
        "start" => AppAction::Start,
        "go" => AppAction::Go,
        "stop" => AppAction::Stop,
        "help" => AppAction::Help,
        _ => return Err(LogApp::get_error(200, &arg)),
      },
    };
    Ok(())
  }
}