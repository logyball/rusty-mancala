use std::{env, process};
mod client;
mod server;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("error: too few args");
    }
    if args[1] == "s" {
        println!("run server");
        server::run_server();
    }
    if args[1] == "c" {
        println!("run client");
        client::run_client();
    }
    process::exit(0);
}
