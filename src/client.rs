use std::io::{self, BufRead, BufReader, Write, Read};
use std::net::TcpStream;
use std::str;

pub fn run_client() {
    let connection = "localhost:42069";
    let mut buffer_arr = [0; 512];
    match TcpStream::connect(connection) {
        Ok(mut stream) => {
            loop {
                let mut input = String::new();
                let mut buffer: Vec<u8> = Vec::new();

                // Read input from user
                io::stdin().read_line(&mut input).expect("I/O error");

                // Exit loop & terminate connection if user enters "quit"
                if input.trim_end().eq_ignore_ascii_case("quit") {
                    println!("Goodbye!");
                    break;
                }

                // Write user input to server
                stream
                    .write_all(input.as_bytes())
                    .expect("Server write error");
                stream.flush().unwrap();

                match stream.read(&mut buffer_arr) {
                    Ok(size) => {
                        if size == 0 {
                            println!("Server terminated connection");
                            break;
                        }
                        let response = str::from_utf8(&buffer_arr[0..size]).unwrap().trim_end();
                        println!("server response: {}", response);
                    }
                    Err(_) => {
                        println!("server did something bad");
                    }
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Connection terminated");
}