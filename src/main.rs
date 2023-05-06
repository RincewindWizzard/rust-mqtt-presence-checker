use crate::config::ApplicationConfig;

mod config;

fn main() {
    let config: ApplicationConfig = ApplicationConfig::load_config().unwrap();
    println!("Hello, world!");
}
