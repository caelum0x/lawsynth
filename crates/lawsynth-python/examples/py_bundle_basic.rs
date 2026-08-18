#[path = "../src/config.rs"]
mod config;
#[path = "../src/error.rs"]
mod error;

use config::PythonConfig;
use error::message as error_message;
fn main() {
    let config = PythonConfig::default();
    println!("Python boundary rejects unknown data: {}", config.reject_unknown_keyword_data);
    println!("error translation: {}", error_message("invalid dataset"));
}
