
// Help struct
pub struct Help {}

impl Help {
  // Display help information
  pub fn help() {
    let desc = "TryTeex is a high-speed FastCGI server for WEB applications written in the RUST programming language.";
    let ver = format!("tryteex version: {}", env!("CARGO_PKG_VERSION").to_owned());
    let help = "
Usage: tryteex [start|stop|help]

Actions:
    start         : start tryteex server
    stop          : stop tryteex server without kill working threads
    help          : this help
";
    println!("");
    println!("{}", desc);
    println!("{}", ver);
    println!("{}", help);
  }
}