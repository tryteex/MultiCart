use std::{fs::OpenOptions, io::Write, process};

use chrono::Local;

// Logging system
pub struct LogApp { 
  pid: u32,           // System PID
  dir: String,        // Directory for the log file
}

impl LogApp {

  // Constructor
  pub fn new() -> LogApp{
    LogApp { 
      pid: 0,
      dir: "".to_owned(),
    }
  }

  // Setting the logging system
  pub fn set(&mut self, pid: u32, dir: String) {
    self.pid = pid;
    self.dir = dir;
  }

  // Write an error to the log file and exit the program
  pub fn exit_err(&self, err: &str) -> ! {
    let file_name = format!("{}/error.log", self.dir);
    let time = Local::now().format("%Y.%m.%d %H:%M:%S%.9f %:z").to_string();
    let str = format!("ID:{} {} {}\n", self.pid, time, err);
    eprint!("{}", &str);
    match OpenOptions::new().create(true).write(true).append(true).open(file_name) {
      Ok(mut file) => file.write_all(str.as_bytes()).unwrap(),
      Err(e) => eprintln!("{}", format!("ID:{} {} {}\n", self.pid, time, LogApp::get_error(2, &e.to_string()))),
    };
    process::exit(1)
  }

  // Getting the text of the error code
  pub fn get_error(err: u32, text: &str) -> String {
    let mut s = String::with_capacity(128);
    s.push_str("Error ");
    s.push_str(&err.to_string());
    match err {
      // Log error
      1 => s.push_str(": Can't write log to file. System message: "),
      2 => s.push_str(": Can't open log file. System message: "),
      
      // Config file error
      100 => s.push_str(": Unknown error when opening config file: "),
      101 => s.push_str(": Value \"max_connection\" must be > 0 in config file"),
      102 => s.push_str(": Unknown value \"max_connection={}\" in config file"),
      103 => s.push_str(": Unknown value \"socket={}\" in config file"),
      104 => s.push_str(": Value \"socket\" mustn't be empty in config file"),
      105 => s.push_str(": Value \"app\" must be > 0 in config file"),
      106 => s.push_str(": Unknown value \"app={}\" in config file"),
      107 => s.push_str(": Value \"dir\" mustn't be empty in config file"),
      108 => s.push_str(": Value length \"dir={}\" must be < 1024 in config file"),
      109 => s.push_str(": Value \"version\" mustn't be empty in config file"),
      110 => s.push_str(": Value length \"version={}\" must be < 12 in config file"),
      111 => s.push_str(": Value \"db_host\" mustn't be empty in config file"),
      112 => s.push_str(": Value \"db_port\" mustn't be empty in config file"),
      113 => s.push_str(": Value \"db_user\" mustn't be empty in config file"),
      114 => s.push_str(": Value \"db_pwd\" mustn't be empty in config file"),
      115 => s.push_str(": Value \"db_name\" mustn't be empty in config file"),
      116 => s.push_str(": Value \"salt\" mustn't be empty in config file"),

      // Action error
      200 => s.push_str(": Unknown command: "),
      201 => s.push_str(": Start server error: "),

      // Command error
      250 => s.push_str(": Can't send command. System error: "),
      251 => s.push_str(": Read empty data from IRC channel."),
      252 => s.push_str(": Unrecognized data read. Error: "),
      253 => s.push_str(": Unrecognized IRC answer. Data: "),
      254 => s.push_str(": Receive error data."),
      255 => s.push_str(": Unrecognized IRC answer. Data: "),
      256 => s.push_str(": Unrecognized IRC answer. Data: "),
      257 => s.push_str(": Unrecognized data read. Error: "),
      258 => s.push_str(": Unrecognized IRC answer. Data: "),
      259 => s.push_str(": Receive error data."),
      260 => s.push_str(": Can't read command from IRC channel. System error: "),
      261 => s.push_str(": Unrecognized IRC answer. Data: "),
      262 => s.push_str(": Receive error data."),
      263 => s.push_str(": Can't send command. System error: "),
      264 => s.push_str(": Permission denied to connect to IRC server"),
      265 => s.push_str(": Connection refused of IRC server"),
      266 => s.push_str(": Connection reset of IRC server"),
      267 => s.push_str(": Connection aborted of IRC server"),
      268 => s.push_str(": Not connected to IRC server"),
      269 => s.push_str(": IP addr not available "),
      270 => s.push_str(": Connection timeout. Maybe server IRC not started "),
      271 => s.push_str(": Connection error: "),
      272 => s.push_str(": Send to stdout error data. Error: "),

      // Start server
      300 => s.push_str(": Permission denied to open IRC socket"),
      301 => s.push_str(": Socket busy for opening IRC socket"),
      302 => s.push_str(": IRC socket not avaibale for opening"),
      303 => s.push_str(": Error open IRC socket. System error: "), 

      // SQL error
      350 => s.push_str(": Error connect to sql server. Error text: "), 
      351 => s.push_str(": Error set time_zone. Error text: "), 

      // Lang
      370 => s.push_str(": Error get langs. Error text: "), 

      // Template
      380 => s.push_str(": Error get templates. Error text: "), 

      // Start fastCGI server
      400 => s.push_str(": Permission denied to open socket"),
      401 => s.push_str(": Socket busy for opening socket"),
      402 => s.push_str(": Socket not avaibale for opening"),
      403 => s.push_str(": Error open socket. System error: "), 

      // Server go
      500 => s.push_str(": The network connection is abruptly disconnected. System error: "),
      501 => s.push_str(": Mix up connections."),
      502 => s.push_str(": Queue is wrong."),

      // Unknown error
      _ => s.push_str(": Unknown error: "),
    };
    s.push_str(text);
    s
  }
}