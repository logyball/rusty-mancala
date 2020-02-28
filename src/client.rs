use std::io::{self, Read, Write};
use std::net::TcpStream;

use crate::proto::*;

fn build_client_msg() -> Option<Msg> {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("I/O error");
    if input.trim_end().eq_ignore_ascii_case("quit") {
        println!("Goodbye!");
        return None
    }
    if input.trim_end().eq_ignore_ascii_case("listgames") {
        return Some(Msg {
            status: Status::Ok,
            headers: Headers::Read,
            command: Commands::ListGames,
            data: String::new()
        })
    }
    if input.trim_end().eq_ignore_ascii_case("listusers") {
        return Some(Msg {
            status: Status::Ok,
            headers: Headers::Read,
            command: Commands::ListUsers,
            data: String::new()
        })
    }
    if input.trim_end().eq_ignore_ascii_case("setnick") {
        print!("input nickname (no funny business): ");
        io::stdout().flush().expect("Client input something nonesensical");
        let mut nickname = String::new();
        io::stdin().read_line(&mut nickname).expect("I/O error");
        return Some(Msg {
            status: Status::Ok,
            headers: Headers::Write,
            command: Commands::SetNick,
            data: nickname.trim_end().parse().unwrap()
        })
    }
    None
}

pub fn run_client() {
    let connection = "localhost:42069";
    let mut buffer_arr = [0; 512];
    match TcpStream::connect(connection) {
        Ok(mut stream) => loop {
            let msg = build_client_msg();
            match msg {
                Some(msg) => {
                    msg.serialize(&mut buffer_arr);
                    stream
                        .write_all(&buffer_arr)
                        .expect("Server write error");
                    stream.flush().unwrap();
                }
                None => {
                    // TODO - send server "kill me" message
                    println!("Goodbye!");
                    break;
                }
            }
            match stream.read(&mut buffer_arr) {
                Ok(size) => {
                    if size == 0 {
                        println!("Server terminated connection");
                        break;
                    }
                    let res_msg : Msg = bincode::deserialize(&buffer_arr[0..size]).unwrap();
                    println!("server response: {:?}", res_msg);
                }
                Err(_) => {
                    println!("server did something bad");
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Connection terminated");
}
