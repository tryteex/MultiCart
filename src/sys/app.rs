use super::{init::Init, log::LogApp};

use std::{process::Command, net::TcpStream, time::Duration, io::{Write, Read}, str::from_utf8};

pub struct App {}

impl App {
  
  // Set the signal
  fn set_control(str: &str, param: &str, init: &Init, log: &LogApp) -> Option<Vec<u8>> {

    // Connection to the server
     match TcpStream::connect_timeout(&init.sys.irc, Duration::from_secs(1)) {
      Ok(mut tcp) => {
        // Format IRC request
        let request = format!("{} {} {}", &init.id, &str, &param);
        // Send the IRC request
        match tcp.write_all(&request.trim().as_bytes()) {
          Ok(()) => {
            // Set reading timeout
            if let Err(e) = tcp.set_read_timeout(Some(Duration::from_secs(30))) {
              log.exit_err(&LogApp::get_error(250, &e.to_string()));
            }
            let mut buffer: Vec<u8> = Vec::new();
            // Reading data
            match tcp.read_to_end(&mut buffer) {
              Ok(size) => match size {
                0 => log.exit_err(&LogApp::get_error(251, "")),
                _ => {
                  // Data received
                  let data = &buffer[..size];
                  // Split data
                  let start = match data[..].iter().position(|&r| r == b' ') {
                    Some(start) => start,
                    None => log.exit_err(&LogApp::get_error(252, "")),
                  };
                  // Get PID of the server
                  let _irc_id = match from_utf8(&data[..start-1]) {
                    Ok(irc_id) => match irc_id {
                      "" => log.exit_err(&LogApp::get_error(254, "")),
                      text => match text.parse::<u16>() {
                        Ok(val) => match val {
                          0 => log.exit_err(&LogApp::get_error(253, text)),
                          id => id,
                        },
                        Err(_) => log.exit_err(&LogApp::get_error(255, text)),
                      },
                    },
                    Err(e) => log.exit_err(&LogApp::get_error(256, &e.to_string())),
                  };
                  // Split data
                  let finish = match data[start+1..].iter().position(|&r| r == b' ') {
                    Some(finish) => start + finish + 1,
                    None => log.exit_err(&LogApp::get_error(257, "")),
                  };
                  // Get and check sended the IRC command
                  match from_utf8(&data[start+1..finish]) {
                    Ok(com) => {
                      if !str.eq(com) { log.exit_err(&LogApp::get_error(259, com)); }
                    },
                    Err(e) => log.exit_err(&LogApp::get_error(258, &e.to_string())),
                  };
                  // Get the answer
                  match from_utf8(&data[finish+1..finish+4]) {
                    Ok(ok) => match ok {
                      "ok:" => {
                        let res = data[finish+4..].to_vec();
                        if res.len() == 0 { return None; }
                        return Some(res);
                      },
                      _ => log.exit_err(&LogApp::get_error(262, ok)),
                    },
                    Err(e) => log.exit_err(&LogApp::get_error(261, &e.to_string())),
                  };
                },
              },
              Err(e) => log.exit_err(&LogApp::get_error(260, &e.to_string())),
            };
          },
          Err(e) => log.exit_err(&LogApp::get_error(263, &e.to_string())),
        };
      },
      Err(e) => match e.kind() {
        std::io::ErrorKind::PermissionDenied => log.exit_err(&LogApp::get_error(264, "")),
        std::io::ErrorKind::ConnectionRefused => log.exit_err(&LogApp::get_error(265, "")),
        std::io::ErrorKind::ConnectionReset => log.exit_err(&LogApp::get_error(266, "")),
        std::io::ErrorKind::ConnectionAborted => log.exit_err(&LogApp::get_error(267, "")),
        std::io::ErrorKind::NotConnected => log.exit_err(&LogApp::get_error(268, "")),
        std::io::ErrorKind::AddrNotAvailable => log.exit_err(&LogApp::get_error(269, &init.sys.irc.to_string())),
        std::io::ErrorKind::TimedOut => log.exit_err(&LogApp::get_error(270, &init.sys.irc.to_string())),
        _ =>log.exit_err(&LogApp::get_error(271, &e.to_string())),
      },
    };
  }

  // Start the server in the background stream and exit
  pub fn start(init: &Init, log: &LogApp) {
    let file = &init.exe;
    match Command::new(file).arg("go").current_dir(&init.dir).spawn() {
        Ok(_) => {},
        Err(e) => log.exit_err(&LogApp::get_error(201, &e.to_string())),
    };
  }

  // Send an IRC "stop" signal and exit
  pub fn stop(init: &Init, log: &LogApp) {
    App::set_control("stop", "", init, log);
  }

}