use std::{net::TcpStream, time::Duration, thread, io::{ErrorKind, Read, Write, Error}, collections::HashMap, sync::Arc};

use byteorder::{BigEndian, ByteOrder};
use cast::{u8, u16, u32, usize};

// FastCGI header
pub struct Header
{
  pub version: u8,
  pub header_type: HeaderType,
  pub request_id: u16,
  pub content_length: u16,
  pub padding_length: u8,
  pub reserved: u8,
}

// FastCGI header type
pub enum HeaderType {
  BeginRequest,
  AbortRequest,
  EndRequest,
  Params,
  Stdin,
  Stdout,
  Stderr,
  Data,
  GetValues,
  GetValuesResult,
  UnknownType,
  Error(u8),
}

// FastCGI role
pub enum Role {
  Responder,
  Authorized,
  Filter,
  Error,
}

// FastCGI Begin request data
pub struct BeginRequest {
  pub role: Role,             // FastCGI Role
  pub flags: u8,              // KeepConnect indication
  pub reserved: [u8;5],
}

// Content of the fastCGI record
pub enum ContentData {
  None,
  BeginRequest(BeginRequest),
  Param(HashMap<String, String>),
  Stream(Vec<u8>),
  Error,
  ErrorStream,
  Break,
  Unknown(Unknown),
  Raw(Arc<Vec<u8>>, usize, u16),
  End(End),
}

// FastCGI record
pub struct Record {
  pub header: Header,
  pub data: ContentData,
}

// Status of the fastCGI record
pub enum ReadStatus {
  Continue,
  Break,
  Next,
  Result(Record),
  ErrorStream, 
}

// Status of reading fastCGI record
pub enum RecordType {
  None,
  Some(Record),
  ErrorStream,
}

// FastCGI Unknown request data
pub struct Unknown {
  pub unknown_type: u8,
  pub reserved: [u8; 7],
}

// FastCGI End request data
pub struct End {
  pub code: u32,
  pub protocol: u8,
  pub reserved: [u8; 3],
}

pub const MS1: std::time::Duration = Duration::from_millis(1);

pub const FASTCGI_VERSION: u8 = 1;

pub const FASTCGI_HEADER_LEN: usize = 8;
pub const FASTCGI_BEGIN_REQUEST_LEN: usize = 8;
pub const FASTCGI_MAX_REQUEST_LEN: usize = 65798;
pub const FASTCGI_MAX_CONTENT_LEN: usize = 65535;
pub const FASTCGI_MAX_GETS_LEN: usize = 63;

pub const FASTCGI_BEGIN_REQUEST: u8 = 1;
pub const FASTCGI_ABORT_REQUEST: u8 = 2;
pub const FASTCGI_END_REQUEST: u8 = 3;
pub const FASTCGI_PARAMS: u8 = 4;
pub const FASTCGI_STDIN: u8 = 5;
pub const FASTCGI_STDOUT: u8 = 6;
pub const FASTCGI_STDERR: u8 = 7;
pub const FASTCGI_DATA: u8 = 8;
pub const FASTCGI_GET_VALUES: u8 = 9;
pub const FASTCGI_GET_VALUES_RESULT: u8 = 10;
pub const FASTCGI_UNKNOWN_TYPE: u8 = 11;

pub const FASTCGI_RESPONDER: u16 = 1;
pub const FASTCGI_AUTHORIZER: u16 = 2;
pub const FASTCGI_FILTER: u16 = 3;

pub const FASTCGI_REQUEST_COMPLETE: u8 = 0;

// Wrapper of FastCGI server
pub struct FastCGI { }

impl FastCGI {
  // Read FastCGI records
  pub fn read_record(seek: &mut usize, size: &mut usize, need_read: &mut bool, buffer: &mut[u8], stream: &mut TcpStream, max_connection: usize) -> RecordType {
    loop{
      if *need_read {
        // Checks indicator to read from the stream buffer
        if *seek > 0 {
          if *seek == *size {
            *size = 0;
            *seek = 0;
          } else {
            buffer.copy_within(*seek.., 0);
            *size -= *seek;
            *seek = 0;
          }
        }
        // Read data
        match stream.read(&mut buffer[*size..]) {
          Ok(n) => {
            if n == 0 {
              return RecordType::ErrorStream;
            };
            *size += n;
            *need_read = false;
            // Read one record
            match FastCGI::read(seek, size, buffer, stream, max_connection) {
              ReadStatus::Continue => {
                *need_read = true;
                continue;
              },
              ReadStatus::Break => return RecordType::None,
              ReadStatus::Result(record) => return RecordType::Some(record),
              ReadStatus::Next => continue,
              ReadStatus::ErrorStream => return RecordType::ErrorStream,
            };
          },
          Err(e) => match e.kind() {
            ErrorKind::Interrupted => {
              *need_read = true;
              continue;
            },
            ErrorKind::WouldBlock => {
              match FastCGI::read(seek, size, buffer, stream, max_connection) {
                ReadStatus::Continue => {
                  thread::sleep(MS1);
                  *need_read = true;
                  continue;
                },
                ReadStatus::Break => return RecordType::None,
                ReadStatus::Result(record) => return RecordType::Some(record),
                ReadStatus::Next => continue,
                ReadStatus::ErrorStream => return RecordType::ErrorStream,
              }
            },
            _ =>break,
          },
        };
      } else {
        match FastCGI::read(seek, size, buffer, stream, max_connection) {
          ReadStatus::Continue => {
            *need_read = true;
            continue;
          },
          ReadStatus::Break => return RecordType::None,
          ReadStatus::Result(record) => return RecordType::Some(record),
          ReadStatus::Next => continue,
          ReadStatus::ErrorStream => return RecordType::ErrorStream,
        };
      }
    }
    RecordType::None
  }

  // Decode one FastCGI record
  fn read(seek: &mut usize, size: &mut usize, buffer: &mut[u8], stream: &mut TcpStream, max_connection: usize) -> ReadStatus {
    if *size - *seek < FASTCGI_HEADER_LEN {
      return ReadStatus::Continue;
    }
    // Read header
    let header = FastCGI::read_header(&buffer[*seek..*seek + FASTCGI_HEADER_LEN]);
    if let None = header {
      return ReadStatus::Break;
    }
    let header = header.unwrap();
    // Validate readed data
    if let HeaderType::BeginRequest | HeaderType::AbortRequest | HeaderType::Params | HeaderType::Stdin | HeaderType::Data = header.header_type {
      if header.request_id == 0 {
        *seek += usize::from(header.content_length) + usize::from(header.padding_length) + FASTCGI_HEADER_LEN;
        return ReadStatus::Break;
      };
    };
    // Second step of Validating readed data
    if let HeaderType::EndRequest | HeaderType::Stdout | HeaderType::Stderr | HeaderType::GetValuesResult | HeaderType::UnknownType = header.header_type {
      *seek += usize::from(header.content_length) + usize::from(header.padding_length) + FASTCGI_HEADER_LEN;
      return ReadStatus::Break;
    }
    if header.content_length == 0 {
      // Check if data is empty 
      *seek += usize::from(header.padding_length) + FASTCGI_HEADER_LEN;
      if let HeaderType::GetValues | HeaderType::BeginRequest = header.header_type {
        return ReadStatus::Break;
      }
      if let HeaderType::Params | HeaderType::AbortRequest | HeaderType::Stdin = header.header_type {
        let record = Record {
          header,
          data: ContentData::None,
        };
        return ReadStatus::Result(record);
      }
    }
    // Check buffer size
    if *size - *seek < usize::from(header.content_length) + usize::from(header.padding_length) {
      return ReadStatus::Continue;
    }
    let hseek = *seek + FASTCGI_HEADER_LEN;
    // Read data
    let data = match header.header_type {
      HeaderType::BeginRequest => {
        if usize::from(header.content_length) != FASTCGI_BEGIN_REQUEST_LEN {
          *seek += usize::from(header.content_length) + usize::from(header.padding_length) + FASTCGI_HEADER_LEN;
          return ReadStatus::Break;
        }
        FastCGI::read_begin_request(&buffer[hseek..hseek + usize::from(header.content_length)])
      },
      HeaderType::Params => FastCGI::read_param(&mut buffer[hseek..hseek + usize::from(header.content_length)]),
      HeaderType::GetValues => FastCGI::read_write_value(header.request_id, stream, max_connection),
      HeaderType::AbortRequest => FastCGI::read_stream(&mut buffer[hseek..hseek + usize::from(header.content_length)]),
      HeaderType::Stdin => FastCGI::read_stream(&mut buffer[hseek..hseek + usize::from(header.content_length)]),
      HeaderType::Data => FastCGI::read_stream(&mut buffer[hseek..hseek + usize::from(header.content_length)]),
      HeaderType::Error(unknown) => FastCGI::write_unknown(unknown, header.request_id, stream),
      _ => ContentData::Error,
    };
    *seek += usize::from(header.content_length) + usize::from(header.padding_length) + FASTCGI_HEADER_LEN;
    if let ContentData::Break | ContentData::Error = data {
      return ReadStatus::Break;
    }
    if let ContentData::ErrorStream = data {
      return ReadStatus::ErrorStream;
    }
    if let ContentData::None = data {
      return ReadStatus::Next;
    }
    let record = Record {
      header,
      data,
    };
    ReadStatus::Result(record)
  }

  // Answer GetValues low-level request
  fn read_write_value(request_id: u16, stream: &mut TcpStream, max_connection: usize) -> ContentData {
    if request_id > 0 {
      return ContentData::None;
    }
    let mut params: HashMap<String, String> = HashMap::with_capacity(3);
    params.insert("FCGI_MAX_CONNS".to_owned(), max_connection.to_string());
    params.insert("FASTCGI_MAX_REQS".to_owned(), max_connection.to_string());
    params.insert("FASTCGI_MAX_CONNS".to_owned(), max_connection.to_string());

    let len = u16(params.len()).unwrap();
    let record = Record {
      header: Header {
        version: FASTCGI_VERSION,
        header_type: HeaderType::GetValuesResult,
        request_id,
        content_length: len,
        padding_length: 0,
        reserved: 0,
      },
      data: ContentData::Param(params),
    };
    let data = FastCGI::record_array(record);
    if let Err(_) = stream.write_all(&data[..]) {
      return ContentData::ErrorStream;
    }
    ContentData::Break
  }

  // Answer unknown command
  fn write_unknown(unknown: u8, request_id: u16, stream: &mut TcpStream) -> ContentData {
    let record = Record {
      header: Header {
        version: FASTCGI_VERSION,
        header_type: HeaderType::UnknownType,
        request_id,
        content_length: 8,
        padding_length: 0,
        reserved: 0,
      },
      data: ContentData::Unknown(Unknown {
        unknown_type: unknown,
        reserved: [0; 7],
      }),
    };

    let data = FastCGI::record_array(record);
    if let Err(_) = stream.write_all(&data[..]) {
      return ContentData::ErrorStream;
    }
    ContentData::Break
  }

  // Read raw data
  fn read_stream(data: &mut [u8]) -> ContentData {
    ContentData::Stream(data.to_vec())
  }

  // Read param
  fn read_param(data: &mut [u8]) -> ContentData {
    let len: usize = data.len();
    if len == 0 {
      return ContentData::Error;
    }
    let mut param: HashMap<String, String> = HashMap::new();
    let mut size: usize = 0;
    while size < len {
      let key_len: usize;
      if (data[size] >> 7) == 0 {
        if size + 1 > len {
          return ContentData::Error;
        }
        key_len = usize::from(data[size]);
        size += 1;
      } else {
        if size + 4 > len {
          return ContentData::Error;
        }
        data[size] = data[size] & 0x7F;
        key_len = BigEndian::read_u32(&data[size..size+4]) as usize;
        size += 4;
      }
      if key_len == 0{
        return ContentData::Error;
      }
      let value_len: usize;
      if (data[size] >> 7) == 0 {
        if size + 1 > len {
          return ContentData::Error;
        }
        value_len = usize::from(data[size]);
        size += 1;
      } else {
        if size + 4 > len {
          return ContentData::Error;
        }
        data[size] = data[size] & 0x7F;
        value_len = BigEndian::read_u32(&data[size..size+4]) as usize;
        size += 4;
      }
      if size + key_len + value_len > len {
        return ContentData::Error;
      }
      let key = Vec::from(&data[size..size + key_len]);
      size += key_len;
      let value = Vec::from(&data[size..size + value_len]);
      size += value_len;

      if let Ok(k) = String::from_utf8(key.clone()) {
        if let Ok(v) = String::from_utf8(value.clone()) {
          param.insert(k, v);
        }
      }
    }
    ContentData::Param(param)
  }

  // Read "begin" data
  fn read_begin_request(data: &[u8]) -> ContentData {
    let request = BeginRequest {
        role: FastCGI::get_role(BigEndian::read_u16(&data[0..2])),
        flags: data[2],
        reserved: [data[3],data[4],data[5],data[6],data[7]],
    };
    if let Role::Error = request.role {
      return ContentData::Error;
    }
    ContentData::BeginRequest(request)
  }

  // Read the header of fastCGI records
  fn read_header(data: &[u8]) -> Option<Header> {
    let header = Header {
      version: data[0],
      header_type: FastCGI::get_header_type(data[1]),
      request_id: BigEndian::read_u16(&data[2..4]),
      content_length: BigEndian::read_u16(&data[4..6]),
      padding_length: data[6],
      reserved: data[7],
    };
    if header.version != FASTCGI_VERSION {
      return None;
    }
    Some(header)
  }

  // Get header type
  fn get_header_type(header_type: u8) -> HeaderType {
    match header_type {
      FASTCGI_BEGIN_REQUEST => HeaderType::BeginRequest,
      FASTCGI_ABORT_REQUEST => HeaderType::AbortRequest,
      FASTCGI_END_REQUEST => HeaderType::EndRequest,
      FASTCGI_PARAMS => HeaderType::Params,
      FASTCGI_STDIN => HeaderType::Stdin,
      FASTCGI_STDOUT => HeaderType::Stdout,
      FASTCGI_STDERR => HeaderType::Stderr,
      FASTCGI_DATA => HeaderType::Data,
      FASTCGI_GET_VALUES => HeaderType::GetValues,
      FASTCGI_GET_VALUES_RESULT => HeaderType::GetValuesResult,
      FASTCGI_UNKNOWN_TYPE => HeaderType::UnknownType,
      _ => HeaderType::Error(header_type),
    }
  }

  // Set header type
  fn set_header_type(header_type: HeaderType) -> u8 {
    match header_type {
      HeaderType::BeginRequest => FASTCGI_BEGIN_REQUEST,
      HeaderType::AbortRequest => FASTCGI_ABORT_REQUEST,
      HeaderType::EndRequest => FASTCGI_END_REQUEST,
      HeaderType::Params => FASTCGI_PARAMS,
      HeaderType::Stdin => FASTCGI_STDIN,
      HeaderType::Stdout => FASTCGI_STDOUT,
      HeaderType::Stderr => FASTCGI_STDERR,
      HeaderType::Data => FASTCGI_DATA,
      HeaderType::GetValues => FASTCGI_GET_VALUES,
      HeaderType::GetValuesResult => FASTCGI_GET_VALUES_RESULT,
      HeaderType::UnknownType => FASTCGI_UNKNOWN_TYPE,
      _ => 0,
    }
  }

  // Get role
  fn get_role(role: u16) -> Role {
    match role {
      FASTCGI_RESPONDER => Role::Responder,
      FASTCGI_AUTHORIZER => Role::Authorized,
      FASTCGI_FILTER => Role::Filter,
      _ => Role::Error,
    }
  }

  // Answer to the WEB server
  pub fn write_response(header: &Header, answer: Vec<u8>, stream: &mut TcpStream) -> Result<(), Error> {
    let mut data: Vec<u8> = Vec::new();
    let len = answer.len();
    let mut size: u16;
    let mut seek: usize = 0;
    let pack = Arc::new(answer);
    // Split data to parts
    while seek < len {
      if seek + FASTCGI_MAX_CONTENT_LEN < len {
        size = u16(FASTCGI_MAX_CONTENT_LEN).unwrap();
      } else {
        size = u16(len - seek).unwrap();
      };
      let record = Record {
        header: Header {
          version: FASTCGI_VERSION,
          header_type: HeaderType::Stdout,
          request_id: header.request_id,
          content_length: size,
          padding_length: 0,
          reserved: 0,
        },
        data: ContentData::Raw(Arc::clone(&pack), seek, size),
      };
      data.extend_from_slice(&FastCGI::record_array(record)[..]);
      seek += usize(size);
    }
    let record = Record {
      header: Header {
        version: FASTCGI_VERSION,
        header_type: HeaderType::Stdout,
        request_id: header.request_id,
        content_length: 0,
        padding_length: 0,
        reserved: 0,
      },
      data: ContentData::None,
    };
    data.extend_from_slice(&FastCGI::record_array(record)[..]);
    let record = Record {
      header: Header {
        version: FASTCGI_VERSION,
        header_type: HeaderType::EndRequest,
        request_id: header.request_id,
        content_length: 8,
        padding_length: 0,
        reserved: 0,
      },
      data: ContentData::End(End{
        code: 0,
        protocol: FASTCGI_REQUEST_COMPLETE,
        reserved: [0; 3],
      }),
    };
    data.extend_from_slice(&FastCGI::record_array(record)[..]);
    if let Err(e) = stream.write_all(&data[..]) {
      return Err(e); 
    }
    Ok(())
  }

  // Write error to WEB server, but RUST don't have error )))
  // pub fn write_error(header: &Header, answer: Vec<u8>, error: Vec<u8>, stream: &mut TcpStream) -> Result<(), Error> {
  //   let mut data: Vec<u8> = Vec::new();
  //   // error
  //   let mut len = answer.len();
  //   let mut size:u16;
  //   let mut seek: usize = 0;
  //   let pack = Arc::new(error);
  //   while seek < len {
  //     if seek + FASTCGI_MAX_CONTENT_LEN < len {
  //       size = u16(FASTCGI_MAX_CONTENT_LEN).unwrap();
  //     } else {
  //       size = u16(len).unwrap();
  //     };
  //     let record = Record {
  //       header: Header {
  //         version: FASTCGI_VERSION,
  //         header_type: HeaderType::Stderr,
  //         request_id: header.request_id,
  //         content_length: size,
  //         padding_length: 0,
  //         reserved: 0,
  //       },
  //       data: ContentData::Raw(Arc::clone(&pack), seek, size),
  //     };
  //     data.extend_from_slice(&FastCGI::record_array(record)[..]);
  //     seek += usize(size);
  //   }
  //   let record = Record {
  //     header: Header {
  //       version: FASTCGI_VERSION,
  //       header_type: HeaderType::Stderr,
  //       request_id: header.request_id,
  //       content_length: 0,
  //       padding_length: 0,
  //       reserved: 0,
  //     },
  //     data: ContentData::None,
  //   };
  //   data.extend_from_slice(&FastCGI::record_array(record)[..]);
  //   // answer
  //   len = answer.len();
  //   seek = 0;
  //   let pack = Arc::new(answer);
  //   while seek < len {
  //     if seek + FASTCGI_MAX_CONTENT_LEN < len {
  //       size = u16(FASTCGI_MAX_CONTENT_LEN).unwrap();
  //     } else {
  //       size = u16(len).unwrap();
  //     };
  //     let record = Record {
  //       header: Header {
  //         version: FASTCGI_VERSION,
  //         header_type: HeaderType::Stdout,
  //         request_id: header.request_id,
  //         content_length: size,
  //         padding_length: 0,
  //         reserved: 0,
  //       },
  //       data: ContentData::Raw(Arc::clone(&pack), seek, size),
  //     };
  //     data.extend_from_slice(&FastCGI::record_array(record)[..]);
  //     seek += usize(size);
  //     data.clear();
  //   }
  //   let record = Record {
  //     header: Header {
  //       version: FASTCGI_VERSION,
  //       header_type: HeaderType::Stdout,
  //       request_id: header.request_id,
  //       content_length: 0,
  //       padding_length: 0,
  //       reserved: 0,
  //     },
  //     data: ContentData::None,
  //   };
  //   data.extend_from_slice(&FastCGI::record_array(record)[..]);
  //   // end
  //   let record = Record {
  //     header: Header {
  //       version: FASTCGI_VERSION,
  //       header_type: HeaderType::EndRequest,
  //       request_id: header.request_id,
  //       content_length: 8,
  //       padding_length: 0,
  //       reserved: 0,
  //     },
  //     data: ContentData::End(End{
  //       code: 0,
  //       protocol: FASTCGI_REQUEST_COMPLETE,
  //       reserved: [0; 3],
  //     }),
  //   };
  //   data.extend_from_slice(&FastCGI::record_array(record)[..]);
  //   if let Err(e) = stream.write_all(&data[..]) {
  //     return Err(e); 
  //   }
  //   Ok(())
  // }

  // Write abore request
  pub fn write_abort(header: &Header, stream: &mut TcpStream) -> Result<(), Error> {
    let record = Record {
      header: Header {
        version: FASTCGI_VERSION,
        header_type: HeaderType::EndRequest,
        request_id: header.request_id,
        content_length: 8,
        padding_length: 0,
        reserved: 0,
      },
      data: ContentData::End(End{
        code: 0,
        protocol: FASTCGI_REQUEST_COMPLETE,
        reserved: [0; 3],
      }),
    };
    let data = FastCGI::record_array(record);
    if let Err(e) = stream.write_all(&data[..]) {
      return Err(e); 
    }
    Ok(())
  }

  // Prepare record for writing 
  fn record_array(record: Record) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(FASTCGI_HEADER_LEN + FASTCGI_MAX_GETS_LEN + 255);
    let mut buf: [u8; 2] = [0; 2];
    let mut buf4: [u8; 4] = [0; 4];

    data.push(record.header.version);
    data.push(FastCGI::set_header_type(record.header.header_type));
    BigEndian::write_u16(&mut buf, record.header.request_id);
    data.extend_from_slice(&buf); 
    BigEndian::write_u16(&mut buf, record.header.content_length);
    data.extend_from_slice(&buf); 
    data.push(0);
    data.push(0);
    match record.data {
      ContentData::Param(params) => {
        let mut key_len: u32;
        let mut value_len: u32;
        let mut buf: [u8; 4] = [0; 4];
        for (key, value) in params { 
          key_len = u32(key.len()).unwrap();
          if key_len < 128 {
            data.push(u8(key_len).unwrap());
          } else {
            key_len = key_len | 0x80000000;
            BigEndian::write_u32(&mut buf, key_len);
            data.extend_from_slice(&buf);
          }
          value_len = u32(value.len()).unwrap();
          if value_len < 128 {
            data.push(u8(value_len).unwrap());
          } else {
            value_len = value_len | 0x80000000;
            BigEndian::write_u32(&mut buf, value_len);
            data.extend_from_slice(&buf);
          }
          data.extend_from_slice(&key.as_bytes()[..]);
          data.extend_from_slice(&value.as_bytes()[..]);
        }
      },
      ContentData::Unknown(unknown) => {
        data.push(unknown.unknown_type);
        data.extend_from_slice(&unknown.reserved[0..7]);
      },
      ContentData::Raw(arr, seek, size) => {
        data.extend_from_slice(&arr[seek..seek + usize(size)]);
      },
      ContentData::End(end) => {
        BigEndian::write_u32(&mut buf4, end.code);
        data.extend_from_slice(&buf4);
        data.push(end.protocol);
        data.extend_from_slice(&end.reserved[0..3]);
      },
      _ => {}
    }
    data
  }
}