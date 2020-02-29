use std::io::{self, Read, Write};
use std::net::TcpStream;

use crate::proto::*;
use crate::client_input_handler::*;

pub fn run_client() {
    let connection = initial_screen();
    let mut buffer_arr = [0; 512];
    let mut setup: bool = false;
    match TcpStream::connect(&connection) {
        Ok(mut stream) => loop {
            if !setup {
                let message = initial_hello_msg();
                message.serialize(&mut buffer_arr);
                    stream
                        .write_all(&buffer_arr)
                        .expect("Server write error");
                    stream.flush().unwrap();
                setup = true;
                println!("did setup!");
            }
            let mut cli_msg: Msg = Msg {
                status: Status::Ok,
                headers: Headers::Write,
                command: Commands::InitSetup,
                game_status: GameStatus::NotInGame,
                data: String::new()
            };
            match stream.read(&mut buffer_arr) {
                Ok(size) => {
                    if size == 0 {
                        println!("Server terminated connection");
                        break;
                    }
                    let res_msg : Msg = bincode::deserialize(&buffer_arr[0..size]).unwrap();
                    println!("server response: {:?}", res_msg);
                    if res_msg.command == Commands::KillClient { break; }
                    match res_msg.game_status {
                        GameStatus::NotInGame => {
                            cli_msg = handle_out_of_game(connection.clone(), "".to_string());
                        }
                        _ => {}
                    }
                }
                Err(_) => { println!("server did something bad"); }
            }
            cli_msg.serialize(&mut buffer_arr);
            stream
                .write_all(&buffer_arr)
                .expect("Server write error");
            stream.flush().unwrap();
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Connection terminated");
}
