#[macro_use]
extern crate log;
extern crate simple_logger;

use std::{env, process};
mod client;
mod proto;
mod server;
mod server_input_handler;
mod client_input_handler;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("error: too few args");
    }
    if args[1] == "s" {
        simple_logger::init().unwrap();
        server::run_server();
    }
    if args[1] == "c" {
        println!("run client");
        client::run_client();
    }
    process::exit(0);
}
