use std::io::{self, Read, Write};
use std::net::TcpStream;

use crate::proto::*;
use crate::client_input_handler::*;

fn initial_setup_for_client(stream: &mut TcpStream, message: &Msg) -> (bool, String) {
    let mut buffer_arr = [0; 512];
    message.serialize(&mut buffer_arr);
    stream
        .write_all(&buffer_arr)
        .expect("Server write error");
    stream.flush().unwrap();
    let mut res_msg = Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::KillMe,
        game_status: GameStatus::NotInGame,
        data: String::new()
    };
    match stream.read(&mut buffer_arr) {
        Ok(size) => {
            if size == 0 {
                println!("Server terminated connection");
                return (false, String::new());
            }
            res_msg = bincode::deserialize(&buffer_arr[0..size]).unwrap();
        }
        Err(_) => {
            println!("server did something bad");
            return (false, String::new());
        }
    }
    println!("did setup!");
    (true, res_msg.data.clone())
}

pub fn run_client() {
    let connection = initial_screen();
    let mut buffer_arr = [0; 512];
    let mut nickname: String = String::new();
    match TcpStream::connect(&connection) {
        Ok(mut stream) => {
            let mut cli_msg = initial_hello_msg();
            let res_tuple = initial_setup_for_client(&mut stream, &cli_msg);
            if !res_tuple.0 {
                println!("Server terminated connection");
                return
            }
            nickname = res_tuple.1.clone();
            cli_msg = handle_out_of_game(&connection, &nickname);
            loop {
                cli_msg.serialize(&mut buffer_arr);
                stream
                    .write_all(&buffer_arr)
                    .expect("Server write error");
                stream.flush().unwrap();
                match stream.read(&mut buffer_arr) {
                    Ok(size) => {
                        if size == 0 {
                            println!("Server terminated connection");
                            break;
                        }
                        let res_msg : Msg = bincode::deserialize(&buffer_arr[0..size]).unwrap();
                        if res_msg.command == Commands::KillClient { break; }
                        cli_msg = handle_server_response(&res_msg, &connection, &mut nickname);
                    }
                    Err(_) => { println!("server did something bad"); }
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Connection terminated");
}
