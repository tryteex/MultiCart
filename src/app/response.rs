
// Redirect header (HTTP Location)
pub struct Location {
  pub url: String,              // Url
  pub permanently: bool,        // Permanently redirect
}

// Response struct
pub struct Response {
  cookie: Option<Cookie>,       // Cookie
  location: Option<Location>,   // Redirect (HTTP Location)
  code: Option<u16>,            // Header code (HTTP code)
  pub css: Vec<String>,         // Addition css script
  pub js: Vec<String>,          // Addition js script
  pub lang: String,             // Current language code
}

// Cookie struct
pub struct Cookie {
  pub key: String,              // Session key
  pub value: String,            // Session value
  pub time: u32,                // Max-Age cookies value
}

impl Response {
  // Constuctor
  pub fn new() -> Response{
    Response {
      code: None,
      cookie: None,
      location: None,
      css: Vec::with_capacity(16),
      js: Vec::with_capacity(16),
      lang: "".to_string(),
    }
  }

  // Set header code for answer
  pub fn set_header_code(&mut self, code: u16) {
    self.code = Some(code);
  }

  // Get header code for answer
  pub fn get_header_code(&self) ->  Option<&u16> {
    self.code.as_ref()
  }

  // Set sookies
  pub fn set_cookie(&mut self, key: String, value: String, time: u32) {
    self.cookie = Some(Cookie { key, value, time });
  }

  // Get cookies
  pub fn get_cookie(&self) -> Option<&Cookie> {
    self.cookie.as_ref()
  }

  // Set redirect
  pub fn set_redirect(&mut self, url: String, permanently: bool) {
    self.location = Some(Location {url: format!("Location: {}", url), permanently, });
  }

  // Get reditrect
  pub fn get_redirect(&self) -> Option<&Location> {
    self.location.as_ref()
  }

  // Get text from http code
  pub fn get_code(code: u16) -> String {
    match code {
      100 => format!("{} Continue", code),
      101 => format!("{} Switching Protocols", code),
      102 => format!("{} Processing", code),
      103 => format!("{} Early Hints", code),
      200 => format!("{} OK", code),
      201 => format!("{} Created", code),
      202 => format!("{} Accepted", code),
      203 => format!("{} Non-Authoritative Information", code),
      204 => format!("{} No Content", code),
      205 => format!("{} Reset Content", code),
      206 => format!("{} Partial Content", code),
      207 => format!("{} Multi-Status", code),
      208 => format!("{} Already Reported", code),
      226 => format!("{} IM Used", code),
      300 => format!("{} Multiple Choices", code),
      301 => format!("{} Moved Permanently", code),
      302 => format!("{} Found", code),
      303 => format!("{} See Other", code),
      304 => format!("{} Not Modified", code),
      305 => format!("{} Use Proxy", code),
      306 => format!("{} (Unused)", code),
      307 => format!("{} Temporary Redirect", code),
      308 => format!("{} Permanent Redirect", code),
      400 => format!("{} Bad Request", code),
      401 => format!("{} Unauthorized", code),
      402 => format!("{} Payment Required", code),
      403 => format!("{} Forbidden", code),
      404 => format!("{} Not Found", code),
      405 => format!("{} Method Not Allowed", code),
      406 => format!("{} Not Acceptable", code),
      407 => format!("{} Proxy Authentication Required", code),
      408 => format!("{} Request Timeout", code),
      409 => format!("{} Conflict", code),
      410 => format!("{} Gone", code),
      411 => format!("{} Length Required", code),
      412 => format!("{} Precondition Failed", code),
      413 => format!("{} Content Too Large", code),
      414 => format!("{} URI Too Long", code),
      415 => format!("{} Unsupported Media Type", code),
      416 => format!("{} Range Not Satisfiable", code),
      417 => format!("{} Expectation Failed", code),
      418 => format!("{} (Unused)", code),
      421 => format!("{} Misdirected Request", code),
      422 => format!("{} Unprocessable Content", code),
      423 => format!("{} Locked", code),
      424 => format!("{} Failed Dependency", code),
      425 => format!("{} Too Early", code),
      426 => format!("{} Upgrade Required", code),
      428 => format!("{} Precondition Required", code),
      429 => format!("{} Too Many Requests", code),
      431 => format!("{} Request Header Fields Too Large", code),
      451 => format!("{} Unavailable For Legal Reasons", code),
      500 => format!("{} Internal Server Error", code),
      501 => format!("{} Not Implemented", code),
      502 => format!("{} Bad Gateway", code),
      503 => format!("{} Service Unavailable", code),
      504 => format!("{} Gateway Timeout", code),
      505 => format!("{} HTTP Version Not Supported", code),
      506 => format!("{} Variant Also Negotiates", code),
      507 => format!("{} Insufficient Storage", code),
      508 => format!("{} Loop Detected", code),
      510 => format!("{} Not Extended (OBSOLETED)", code),
      511 => format!("{} Network Authentication Required", code),
      code => format!("{} Unassigned", code),
    }
  }
}

