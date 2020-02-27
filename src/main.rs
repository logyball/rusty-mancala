use std::{env, process};
mod server;
mod client;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("error: too few args");
    }
    if args[1] == "s".to_string() {
        println!("run server");
        server::run_server();
    }
    if args[1] == "c".to_string() {
        println!("run client");
        client::run_client();
    }
    process::exit(0);
}
