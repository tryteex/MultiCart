use std::{thread, sync::{Arc, Mutex, mpsc, RwLock, MutexGuard}, net::TcpStream, collections::HashMap, cell::RefCell, rc::Rc};

use postgres::{Client, NoTls};
use postgres_protocol::escape::escape_literal;
use cast::u8;

use crate::sys::log::LogApp;

use super::{go::Go, fastcgi::{Record, FASTCGI_MAX_REQUEST_LEN, FastCGI, RecordType, HeaderType, ContentData}, sys::Sys, i18n::{LangItem, I18n}};

// Message to threads
pub enum Message {
  Terminate,          // Stop all threads
  Job(TcpStream),     // Accept fastCGI connection from the WEB server
}

// Status of the fastCGI connection
#[derive(PartialEq)]
pub enum Status {
  None,               // Nothing or Init
  Begin,              // Receive a "Begin" request
  Param,              // Receive a "Param" request
  ParamEnd,           // Receive a empty "Param" request
  Stdin,              // Receive a "Stdin" request
  Work,               // Receive a empty "Stdin" request and start CRM system
  End,                // Finish
} 

// Worker of thread
pub struct Worker {
  pub id: usize,                                // ID Worker
  pub go: Arc<Mutex<Go>>,                       // Main struct
  pub start: bool,                              // Worker is started
  pub stop: bool,                               // Send the "stop" signal
  pub thread: Option<thread::JoinHandle<()>>,   // Thread 
  pub status: Status,                           // Status for the Worker
}

impl Worker {
  // Constructor
  pub fn new(id: usize, go: Arc<Mutex<Go>>, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Arc<Mutex<Worker>> {
    let conn;
    let max_connection: usize;
    let tz;
    // Init variables
    {
      let g = Mutex::lock(&go).unwrap();
      let init = RwLock::read(&g.init).unwrap();
      max_connection = init.sys.max_connection.into();
      let db = &init.db;
      tz = format!("SET timezone TO {};", escape_literal(&init.time_zone));
      conn = format!("host='{}' port='{}' dbname='{}' user='{}' password='{}' connect_timeout=2 application_name='{} {}' options='--client_encoding=UTF8'", db.host, &db.port, &db.name, &db.user, &db.pwd, &env!("CARGO_PKG_NAME").to_string(), &env!("CARGO_PKG_VERSION").to_string());
    }
    // Connect to the database
    let mut sql = match Client::connect(&conn, NoTls) {
        Ok(sql) => sql,
        Err(e) => {
          let go = Mutex::lock(&go).unwrap();
          let log = RwLock::read(&go.log).unwrap();
          log.exit_err(&LogApp::get_error(350, &e.to_string()));
        },
    };
    // Set timezone
    if tz.len() > 0 {
      if let Err(e) = sql.query(&tz, &[]) {
        let go = Mutex::lock(&go).unwrap();
        let log = RwLock::read(&go.log).unwrap();
        log.exit_err(&LogApp::get_error(351, &e.to_string()));
      };
    }
    // Load enable languages and translates
    {
      // Reading data will take place in one thread, others will wait.
      // When the data is read, other threads will not re-read it.
      let g = Mutex::lock(&go).unwrap();
      let init = RwLock::read(&g.init).unwrap();
      let mut i18n = Mutex::lock(&g.i18n).unwrap();
      // Indication of read data
      if !i18n.load {
        // Load enable languages
        let text = format!("SELECT lang_id, lang, code, name FROM lang WHERE enable");
        match sql.query(&text, &[]) {
          Ok(res) => {
            for row in res {
              let lang_id: i64 = row.get(0);
              let lang_code: String = row.get(1);
              let code: String = row.get(2);
              let name: String = row.get(3);
              let lang_id = u8(lang_id).unwrap();
              i18n.langs_code.insert(lang_code.clone(), lang_id);
              i18n.langs.insert(lang_id, LangItem {code, lang: lang_code, name, });
            }
          },
          Err(e) => {
            let log = RwLock::read(&g.log).unwrap();
            log.exit_err(&LogApp::get_error(351, &e.to_string()));
          },
        };
        // Read translates
        if let Err(e) = i18n.load_lang(&init.dir) {
          let log = RwLock::read(&g.log).unwrap();
          log.exit_err(&LogApp::get_error(352, &e.to_string()));
        };
        // Set indication of read data
        i18n.load = true;
      }
    }
    let go_panic = Arc::clone(&go);
    // Init Worker
    let worker = Worker {
      id,
      go,
      start: false,
      stop: false,
      thread: None,
      status: Status::None,
    };
    let worker = Arc::new(Mutex::new(worker));
    let worker_thread = Arc::clone(&worker);
    // Start the thread
    let thread = thread::spawn(move || {
      let sql = Rc::new(RefCell::new(sql));
      let i18n_prepare: Rc<RefCell<HashMap<u8, HashMap<String, HashMap<String, Rc<RefCell<HashMap<String, String>>>>>>>>;
      // Init variable for translations
      {
        let w = Mutex::lock(&worker_thread).unwrap();
        let g = Mutex::lock(&w.go).unwrap();
        let lang_lock = Mutex::lock(&g.i18n).unwrap();
        i18n_prepare = Rc::new(RefCell::new(Worker::prepare_lang(lang_lock)));
      }
      // Start the thread in an endless cycle
      loop {
        let mut begin_record: Option<Record> = None;
        let mut param_record: HashMap<String, String> = HashMap::with_capacity(128);
        let mut stdin_record: Option<Vec<u8>> = None;
        let wait = Mutex::lock(&receiver).unwrap();
        // Waiting to receive a WEB server connection signal
        match wait.recv() {
          // WEB server is connected

          // Check message to thread
          Ok(message) => match message {
            Message::Job(stream) => {
              // Run fastcgi connection
              Worker::fastcgi_connection(Arc::clone(&worker_thread), stream, max_connection, Rc::clone(&sql), &mut begin_record, &mut param_record, &mut stdin_record, Rc::clone(&i18n_prepare));
              let mut w = Mutex::lock(&worker_thread).unwrap();
              w.start = false;
              w.status = Status::None;
              let mut g = Mutex::lock(&w.go).unwrap();
              g.use_connection -= 1;
            },
            Message::Terminate => break,
          },
          Err(e) => {
            let go = Mutex::lock(&go_panic).unwrap();
            let log = RwLock::read(&go.log).unwrap();
            log.exit_err(&LogApp::get_error(500, &e.to_string()));
          },
        }
      };
    });
    {
      let mut w = Mutex::lock(&worker).unwrap();
      w.thread = Some(thread);
    }
    worker
  }

  // Prepare lang for thread
  fn prepare_lang(i18n: MutexGuard<I18n>) -> HashMap<u8, HashMap<String, HashMap<String, Rc<RefCell<HashMap<String, String>>>>>> {
    let mut data: HashMap<u8, HashMap<String, HashMap<String, Rc<RefCell<HashMap<String, String>>>>>> = HashMap::with_capacity(i18n.data.len());
    for (lang_id, modules) in &i18n.data {
      let mut vl: HashMap<String, HashMap<String, Rc<RefCell<HashMap<String, String>>>>> = HashMap::with_capacity(modules.len());
      for (module, classes) in modules {
        let mut v: HashMap<String, Rc<RefCell<HashMap<String, String>>>> = HashMap::with_capacity(classes.len());
        for (class, map) in classes {
          let mut m: HashMap<String, String> = HashMap::with_capacity(map.len());
          for (key, val) in map {
            m.insert(key.to_string(), val.to_string());
          }
          v.insert(class.to_string(), Rc::new(RefCell::new(m)));
        }
        vl.insert(module.to_string(), v);
      }
      data.insert(*lang_id, vl);
    }
    data
  }

  // Wait terminating of thread
  pub fn join(worker: Arc<Mutex<Worker>>) {
    let thread;
    {
      let mut w = Mutex::lock(&worker).unwrap();
      thread = w.thread.take();
    }
    
    if let Some(main) = thread {
      main.join().unwrap();
    }
  }

  // Start fastCGI connection
  pub fn fastcgi_connection(
    worker: Arc<Mutex<Worker>>, 
    mut stream: TcpStream, 
    max_connection: usize, 
    sql: Rc<RefCell<Client>>, 
    begin_record: &mut Option<Record>, 
    param_record: &mut HashMap<String, String>, 
    stdin_record: &mut Option<Vec<u8>>,
    i18n_prepare: Rc<RefCell<HashMap<u8, HashMap<String, HashMap<String, Rc<RefCell<HashMap<String, String>>>>>>>>
  ){
    let mut buffer: [u8; FASTCGI_MAX_REQUEST_LEN] = [0; FASTCGI_MAX_REQUEST_LEN];
    let mut seek: usize = 0;
    let mut size: usize = 0;
    let mut need_read = true;
    
    // Read data from the WEB server in the loop
    loop {
      // Check stop command
      {
        let w = Mutex::lock(&worker).unwrap();
        if w.stop {
          break;
        }
      }
      // Read one command from the WEB server
      let record = match FastCGI::read_record(&mut seek, &mut size, &mut need_read, &mut buffer[..], &mut stream, max_connection) {
        RecordType::None => continue,
        RecordType::Some(record) => record,
        RecordType::ErrorStream => break,
      };

      // This command must go in a certain order
      match record.header.header_type {
        HeaderType::BeginRequest => {
          // Got "Begin" record
          *begin_record = Some(record);
          let mut w = Mutex::lock(&worker).unwrap();
          if Status::None != w.status {
            break;
          }
          w.status = Status::Begin;
        },
        HeaderType::AbortRequest => {
          // Got "Abort" record
          if let Some(record) = begin_record {
            FastCGI::write_abort(&record.header, &mut stream).unwrap_or(());
          }
          break
        },
        HeaderType::Params => {
          // Got "Param" record
          {
            let w = Mutex::lock(&worker).unwrap();
            match w.status {
              Status::Begin | Status::Param => {},
              _ => break,
            }
          }
          match record.data {
            ContentData::Param(data) => {
              if param_record.len() == 0 {
                *param_record = data;
              } else {
                for (key, value) in data {
                  param_record.insert(key, value);
                }
              } 
              {
                let mut w = Mutex::lock(&worker).unwrap();
                w.status = Status::Param;
              }
            },
            ContentData::None => {
              {
                let mut w = Mutex::lock(&worker).unwrap();
                w.status = Status::ParamEnd;
              }
            },
            _ => break, 
          }
        },
        HeaderType::Stdin => {
          // Got "Stdin" record
          {
            let w = Mutex::lock(&worker).unwrap();
            match w.status {
              Status::Begin | Status::ParamEnd | Status::Stdin => {},
              _ => break,
            }
          }
          match record.data {
            ContentData::Stream(data) => {
              match stdin_record {
                Some(param) => {
                  param.extend_from_slice(&data[..]);
                },
                None => {
                  *stdin_record = Some(data);
                },
              } 
              {
                let mut w = Mutex::lock(&worker).unwrap();
                w.status = Status::Stdin;
              }
            },
            ContentData::None => {
              // Got empty "Stdin" record, so we start the CRM
              {
                let mut w = Mutex::lock(&worker).unwrap();
                w.status = Status::Work;
              }
              // Start CRM
              let answer = Sys::run(Arc::clone(&worker), Rc::clone(&sql), param_record, Rc::clone(&i18n_prepare));
              {
                let mut w = Mutex::lock(&worker).unwrap();
                w.status = Status::End;  
              }
              // Write ansewer to the WEB server
              if let Some(record) = begin_record {
                FastCGI::write_response(&record.header, answer, &mut stream).unwrap_or(());
              }
              // Check "KeepConnect" status
              if let ContentData::BeginRequest(data) = &record.data {
                if data.flags == 0 {
                  // "KeepConnect" was not set
                  break;
                }
                // "KeepConnect" was set
              } else {
                break;
              }
              
              // Clear Worker
              *begin_record = None;
              param_record.clear();
              *stdin_record = None;
              {
                let mut w = Mutex::lock(&worker).unwrap();
                w.start = false;
                w.status = Status::None;
              }
              need_read = true;
            },
            _ => break,
          }
        },
        _ => {},
      };
    }
  }
}
