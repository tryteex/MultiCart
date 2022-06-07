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
      dir: "".to_string(),
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
    match err {
      // Log error
      1 => format!("Error {}: Can't write log to file. System message: {}", err, text),
      2 => format!("Error {}: Can't open log file. System message: {}", err, text),
      3 => format!("Error {}: Can't get current dir", err),
      4 => format!("Error {}: Can't get current dir. System message: {}", err, text),
      5 => format!("Error {}: Can't get current exe. System message: {}", err, text),
      6 => format!("Error {}: Can't get current exe", err),
      
      // Config file error
      100 => format!("Error {}: Unknown error when opening config file \"{}\"", err, text),
      101 => format!("Error {}: Value \"max_connection\" must be > 0 in config file", err),
      102 => format!("Error {}: Unknown value \"max_connection={}\" in config file", err, text),
      103 => format!("Error {}: Unknown value \"socket={}\" in config file", err, text),
      104 => format!("Error {}: Value \"socket\" mustn't be empty in config file", err),
      105 => format!("Error {}: Value \"app\" must be > 0 in config file", err),
      106 => format!("Error {}: Unknown value \"app={}\" in config file", err, text),
      107 => format!("Error {}: Value \"dir\" mustn't be empty in config file", err),
      108 => format!("Error {}: Value length \"dir={}\" must be < 1024 in config file", err, text),
      109 => format!("Error {}: Value \"version\" mustn't be empty in config file", err),
      110 => format!("Error {}: Value length \"version={}\" must be < 12 in config file", err, text),
      111 => format!("Error {}: Value \"db_host\" mustn't be empty in config file", err),
      112 => format!("Error {}: Value \"db_port\" mustn't be empty in config file", err),
      113 => format!("Error {}: Value \"db_user\" mustn't be empty in config file", err),
      114 => format!("Error {}: Value \"db_pwd\" mustn't be empty in config file", err),
      115 => format!("Error {}: Value \"db_name\" mustn't be empty in config file", err),
      116 => format!("Error {}: Value \"salt\" mustn't be empty in config file", err),

      // Action error
      200 => format!("Error {}: Unknown command \"{}\"", err, text),
      201 => format!("Error {}: Start server error: {}", err, text),

      // Command error
      250 => format!("Error {}: Can't send command. System error: {}", err, text),
      251 => format!("Error {}: Read empty data from IRC channel.", err),
      252 => format!("Error {}: Unrecognized data read. Error: {}", err, text),
      253 => format!("Error {}: Unrecognized IRC answer. Data: {}", err, text),
      254 => format!("Error {}: Receive error data.", err),
      255 => format!("Error {}: Unrecognized IRC answer. Data: {}", err, text),
      256 => format!("Error {}: Unrecognized IRC answer. Data: {}", err, text),
      257 => format!("Error {}: Unrecognized data read. Error: {}", err, text),
      258 => format!("Error {}: Unrecognized IRC answer. Data: {}", err, text),
      259 => format!("Error {}: Receive error data.", err),
      260 => format!("Error {}: Can't read command from IRC channel. System error: {}", err, text),
      261 => format!("Error {}: Unrecognized IRC answer. Data: {}", err, text),
      262 => format!("Error {}: Receive error data.", err),
      263 => format!("Error {}: Can't send command. System error: {}", err, text),
      264 => format!("Error {}: Permission denied to connect to IRC server", err),
      265 => format!("Error {}: Connection refused of IRC server", err),
      266 => format!("Error {}: Connection reset of IRC server", err),
      267 => format!("Error {}: Connection aborted of IRC server", err),
      268 => format!("Error {}: Not connected to IRC server", err),
      269 => format!("Error {}: IP addr not available {}", err, text),
      270 => format!("Error {}: Connection timeout. Maybe server IRC {} not started", err, text),
      271 => format!("Error {}: Connection error: {}", err, text),
      272 => format!("Error {}: Send to stdout error data. Error: ", err),

      // Start server
      300 => format!("Error {}: Permission denied to open IRC socket", err),
      301 => format!("Error {}: Socket busy for opening IRC socket", err),
      302 => format!("Error {}: IRC socket not avaibale for opening", err),
      303 => format!("Error {}: Error open IRC socket. System error: {}", err, text), 

      // SQL error
      350 => format!("Error {}: Error connect to sql server. Error text: {}", err, text), 
      351 => format!("Error {}: Error set time_zone. Error text: {}", err, text), 
      352 => format!("Error {}: Error get langs. Error text: {}", err, text), 

      // Start fastCGI server
      400 => format!("Error {}: Permission denied to open socket", err),
      401 => format!("Error {}: Socket busy for opening socket", err),
      402 => format!("Error {}: Socket not avaibale for opening", err),
      403 => format!("Error {}: Error open socket. System error: {}", err, text), 

      // Server go
      500 => format!("Error {}: The network connection is abruptly disconnected. System error: {}", err, text),
      501 => format!("Error {}: Mix up connections.", err),

      // Unknown error
      _ => format!("Error {}: Unknown error: {}", err, text),
    }
  }
}