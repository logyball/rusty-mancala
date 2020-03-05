#[macro_use]
extern crate log;
extern crate simple_logger;
use crate::client::run_client;
use crate::server::run_server;
use clap::{App, Arg};
use log::Level;
use std::process;

mod client;
mod client_input_handler;
mod constants;
mod game_objects;
mod proto;
mod server;
mod server_input_handler;

fn main() {
    let mut client: bool = false;
    let mut server: bool = false;
    let mut port_int: u32 = 4567;
    let matches = App::new("MyApp")
        .version("1.0")
        .author("Logan Ballard, Belén Bustamante")
        .about("Play mancala via TCP")
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .value_name("PORT")
                .help("runs server, specifies port number")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("client")
                .short("c")
                .long("client")
                .help("Runs the client"),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .multiple(true)
                .long("debug")
                .help("Set logging level: null = error, d = warning, d = info, d = everything"),
        )
        .get_matches();

    if let Some(s) = matches.value_of("server") {
        match s.trim().parse() {
            Ok(x) => {
                println!("parsing port");
                port_int = x;
            }
            Err(e) => {
                error!("could not make port into an int: {}!", e);
                process::exit(1);
            }
        }
        match matches.occurrences_of("debug") {
            0 => simple_logger::init_with_level(Level::Error).unwrap(),
            1 => simple_logger::init_with_level(Level::Warn).unwrap(),
            2 => simple_logger::init_with_level(Level::Info).unwrap(),
            3 | _ => simple_logger::init_with_level(Level::Debug).unwrap(),
        }
        server = true;
    }

    if matches.is_present("client") {
        client = true;
    }

    if client && server {
        error!("cant run client and server simultaneously");
        process::exit(1);
    } else if client {
        run_client();
    } else if server {
        run_server(port_int);
    }
    process::exit(0);
}
