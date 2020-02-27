use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str;

pub fn run_client() {
    let connection = "localhost:42069";
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

                // Create read stream & read input from server
                let mut reader = BufReader::new(&stream);
                reader.read_until(b'\n', &mut buffer).expect("Buffer error");
                print!("{}", str::from_utf8(&buffer).expect("Buffer->String error"));
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Connection terminated");
}