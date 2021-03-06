use std::{net::{TcpListener, TcpStream, Shutdown}, time::Duration, thread::{JoinHandle, self}, io::{Read, Write, ErrorKind}, sync::mpsc};
use std::{sync::{Arc, Mutex, RwLock}};

use crate::sys::{init::Init, log::LogApp};

use super::{worker::{Worker, Message}, storage::Storage, i18n::I18n, template::Template, queue::Queue};

pub const MS1: std::time::Duration = Duration::from_millis(1);
// Main struct for program
pub struct Go {
  pub init: Arc<RwLock<Init>>,                                      // Init system
  pub log: Arc<RwLock<LogApp>>,                                     // Log system
  pub tcp: Option<JoinHandle<()>>,                                  // TCP reciever
  pub main: Option<JoinHandle<()>>,                                 // Main thread
  stop: bool,                                                       // Send the "stop" signal
  max_connection: usize,                                            // Max threads or max connections (it is the same) from the WEB server
  pub use_connection: usize,                                        // How many threads are already running
  connections: Vec<(Arc<Mutex<Worker>>, mpsc::Sender<Message>)>,    // Connections from the WEB server
  pub storage: Arc<Mutex<Storage>>,                                 // Memory cache system
  pub i18n: Arc<Mutex<I18n>>,                                       // Translations
  pub tpl: Arc<Mutex<Template>>,                                    // Templates system
  queue: Arc<Mutex<Queue>>,                                         // Input connections
}

impl Go {
  // Start fastCGI and CRM server
  pub fn start(init: Arc<RwLock<Init>>, log: Arc<RwLock<LogApp>>) {
    let init_read = RwLock::read(&init).unwrap();
    let log_read = RwLock::read(&log).unwrap();

    let max_connection = usize::from(init_read.sys.max_connection);

    let q = Queue::new(65536);

    // Create main struct
    let go = Go {
      init: Arc::clone(&init),
      log: Arc::clone(&log),
      tcp: None,
      main: None,
      stop: false,
      max_connection,
      use_connection: 0,
      connections: Vec::with_capacity(max_connection),
      storage: Arc::new(Mutex::new(Storage::new())),
      i18n: Arc::new(Mutex::new(I18n::new())),
      tpl: Arc::new(Mutex::new(Template::new())),
      queue: Arc::new(Mutex::new(q)),
    };

    let go = Arc::new(Mutex::new(go));
   
    // Create threads
    for i in 0..max_connection {
      let (sender, receiver) = mpsc::channel();
      let receiver = Arc::new(Mutex::new(receiver));

      // Start workers
      let w = Worker::new(i, Arc::clone(&go), Arc::clone(&receiver));
      {
        let mut g = Mutex::lock(&go).unwrap();
        g.connections.push((w, sender));
      }
    }

    // Run main thread
    Go::main(Arc::clone(&go));
    // Start threads to listenning to the connections
    Go::open(Arc::clone(&go));

    // Bind IRC channel
    let irc = match TcpListener::bind(&init_read.sys.irc){
      Ok(irc) => irc,
      Err(e) => match e.kind() {
        ErrorKind::PermissionDenied => log_read.exit_err(&LogApp::get_error(300, "")),
        ErrorKind::AddrInUse => log_read.exit_err(&LogApp::get_error(301, "")),
        ErrorKind::AddrNotAvailable => log_read.exit_err(&LogApp::get_error(302, "")),
        _ => log_read.exit_err(&LogApp::get_error(303, &e.to_string())),
      },
    };

    // Set Non_blocking status
    if let Ok(()) = irc.set_nonblocking(true) {
      let ms300 = Duration::from_millis(300);
      // Wait incomming IRC command
      for stream in irc.incoming() {
        match stream {
          // Run command
          Ok(mut stream) => match Go::run_command(Arc::clone(&go), &mut stream) {
            Some(()) => {},
            None => break,
          },
          Err(e) => match e.kind() {
            ErrorKind::WouldBlock => {
              thread::sleep(ms300);
              continue;
            },
            _ => continue,
          },
        };
      }
    }
  }

  // Run IRC command
  fn run_command(go: Arc<Mutex<Go>>, stream: &mut TcpStream) -> Option<()> {
    // Set timeout
    if let Err(_) = stream.set_read_timeout(Some(Duration::from_secs(1))) {
      if let Err(_) = stream.shutdown(Shutdown::Both) { }
      return Some(());
    }
    let mut buffer: [u8; 1024] = [0; 1024];
    // Read and decode command
    match stream.read(&mut buffer) {
      Ok(size) => match size {
        0 => {
          if let Err(_) = stream.shutdown(Shutdown::Both) { }
          return Some(());
        },
        _ => {
          let data = &buffer[..size];
          match std::str::from_utf8(&data) {
            Ok(data) => {
              let mut com = data.split(' ').enumerate();
              let _irc_id = match com.next() {
                Some(irc_id) => match irc_id.1 {
                  "" => return Some(()),
                  text => match text.parse::<u16>() {
                    Ok(val) => match val {
                      0 => return Some(()),
                      id => id,
                    },
                    Err(_) => return Some(()),
                  }
                },
                None => return Some(()),
              };
              // Detecting the command
              match com.next() {
                Some(str) => match str.1 {
                  "stop" => {
                    // Found stop
                    Go::stop(Arc::clone(&go));
                    Go::send_answer(Arc::clone(&go), "stop", stream);
                    return None; 
                  },
                  _ => return Some(()),
                },
                None => return Some(()),
              }
            },
            Err(_) => {
              if let Err(_) = stream.shutdown(Shutdown::Both) { }
              return Some(());
            },
          }
        },
      },
      Err(_) => return Some(()),
    };
  }

  // Stop fastCGI and CRM server
  fn stop(go: Arc<Mutex<Go>>) {
    let tcp_read;
    let main_read;
    // Send "stop" to all threads
    {
      let mut g = Mutex::lock(&go).unwrap();
      g.stop = true;
      tcp_read = g.tcp.take();
      main_read = g.main.take();
      for i in 0..g.max_connection {
        let (item, sender) = g.connections.get(i).unwrap();
        {
          let mut w = Mutex::lock(item).unwrap();
          w.stop = true;  
        }
        sender.send(Message::Terminate).unwrap()
      }
    }
    // Wait while threads aren't stop
    {
      let g = Mutex::lock(&go).unwrap();
      for i in 0..g.max_connection {
        let (item, _) = g.connections.get(i).unwrap();
        Worker::join(Arc::clone(item));
      }
    }
    if let Some(tcp) = tcp_read {
      tcp.join().unwrap();
    }
    if let Some(main) = main_read {
      main.join().unwrap();
    }
  }

  // Send IRC answer
  fn send_answer(go: Arc<Mutex<Go>>, str: &str, stream: &mut TcpStream) {
    let answer;
    {
      let g = Mutex::lock(&go).unwrap();
      let init_read = RwLock::read(&g.init).unwrap();
      answer = format!("{} {} ok:", init_read.id, str);
    }
    if let Err(_) = stream.write_all(&answer.into_bytes()) { }
  }

  // Main loop to recieve tcp connection from WEB server
  pub fn open(go: Arc<Mutex<Go>>) {
    let move_go = Arc::clone(&go);
    // Start thread for listening connections from WEB server
    let tcp = thread::spawn(move || {
      let bind;
      // Bind the connection
      {
        let g = Mutex::lock(&move_go).unwrap();
        let init_read = RwLock::read(&g.init).unwrap();
        bind = TcpListener::bind(&init_read.sys.socket[..]);
      }
      let bind = match bind{
        Ok(bind) => bind,
        Err(e) => match e.kind() {
          ErrorKind::PermissionDenied => {
            let g = Mutex::lock(&move_go).unwrap();
            let log_read = RwLock::read(&g.log).unwrap();
            log_read.exit_err(&LogApp::get_error(400, ""));
          },
          ErrorKind::AddrInUse => {
            let g = Mutex::lock(&move_go).unwrap();
            let log_read = RwLock::read(&g.log).unwrap();
            log_read.exit_err(&LogApp::get_error(401, ""));
          },
          ErrorKind::AddrNotAvailable => {
            let g = Mutex::lock(&move_go).unwrap();
            let log_read = RwLock::read(&g.log).unwrap();
            log_read.exit_err(&LogApp::get_error(402, ""));
          },
          _ => {
            let g = Mutex::lock(&move_go).unwrap();
            let log_read = RwLock::read(&g.log).unwrap();
            log_read.exit_err(&LogApp::get_error(403, &e.to_string()));
          },
        },
      };
  
      // Set Non blocking mode 
      if let Ok(()) = bind.set_nonblocking(true) {
        // Main part on loop. Wait incomming request from WEB server
        for stream in bind.incoming() {
          // Check the stop
          {
            let g = Mutex::lock(&move_go).unwrap();
            if g.stop == true {
              break;
            }
          }
          match stream {
            Ok(stream) => {
              let mut str = stream;
              let mut queue;
              loop {
                {
                  let g = Mutex::lock(&move_go).unwrap(); 
                  if g.stop == true {
                    break;
                  }
                  queue = Arc::clone(&g.queue);
                }
                {
                  let mut q = Mutex::lock(&queue).unwrap(); 
                  match q.push(str) {
                    Some(s) => str = s,
                    None => break,
                  }
                }
                thread::sleep(MS1);
              }
            },
            Err(e) => match e.kind() {
              ErrorKind::WouldBlock => {
                {
                  let g = Mutex::lock(&move_go).unwrap();
                  if g.stop == true {
                    break;
                  }
                }
                thread::sleep(MS1);
              },
              _ => {},
            },
          };
        }
      }
    });
    let mut g = Mutex::lock(&go).unwrap();
    g.tcp = Some(tcp);
  }

  // Main loop to strating fastCGI and CRM server
  pub fn main(go: Arc<Mutex<Go>>) {
    let move_go = Arc::clone(&go);
    // Start thread for listening connections from WEB server
    let mut wait = false;
    let main = thread::spawn(move || loop {
      if wait {
        thread::sleep(MS1);
      }
      let tcp;
      let queue;
      {
        let g = Mutex::lock(&move_go).unwrap();
        if g.stop == true {
          break;
        }
        queue = Arc::clone(&g.queue);
      }
      wait = false;
      {
        let mut q = Mutex::lock(&queue).unwrap();
        if q.empty() {
          wait = true;
          continue;
        } else {
          if let Some(t) = q.take() {
            tcp = t;
          } else {
            let g = Mutex::lock(&move_go).unwrap();
            let log_read = RwLock::read(&g.log).unwrap();
            log_read.exit_err(&LogApp::get_error(502, ""));  
          }
        }
      }
      let mut index: Option<usize>;
      let mut use_connection;
      let max_connection;
      {
        let g = Mutex::lock(&move_go).unwrap();
        max_connection = g.max_connection;
      }
      loop {
        index = None;
        {
          let g = Mutex::lock(&move_go).unwrap();
          if g.stop == true {
            break;
          }
          use_connection = g.use_connection;
        }
        // Wait free thread
        if use_connection < max_connection {
          {
            let mut g = Mutex::lock(&move_go).unwrap();
            g.use_connection += 1;  
          }
          // Find thread
          for i in 0..max_connection {
            let g = Mutex::lock(&move_go).unwrap();
            let (item, _) = g.connections.get(i).unwrap();
            {
              let mut w = Mutex::lock(item).unwrap();
              if w.start == false {
                w.start = true;
                w.count = 0;
                index = Some(i);
                break;
              }
            }
          }
          if let None = index {
            let g = Mutex::lock(&move_go).unwrap();
            let log_read = RwLock::read(&g.log).unwrap();
            log_read.exit_err(&LogApp::get_error(501, ""));
          }
        }
        // If we found the free thread 
        // We send signal for this sleeping thread
        if let Some(ind) = index {
          let g = Mutex::lock(&move_go).unwrap();
          let (_, sender) = g.connections.get(ind).unwrap();
          sender.send(Message::Job(tcp)).unwrap();
          break;
        }
        thread::sleep(MS1);
      }
    });
    let mut g = Mutex::lock(&go).unwrap();
    g.main = Some(main);
  }
}
