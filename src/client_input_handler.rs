#![feature(io)]

use std::io;
use std::io::prelude::*;
use std::fmt::Display;

use crate::proto::*;

// --------------- out of game --------------- //
pub fn initial_screen() -> String {
    let mut stdin = io::stdin();
    let mut host = String::new();
    let mut port = String::new();
    print!("Enter a host: ");
    io::stdout().flush();
    stdin.read_line(&mut host);
    print!("Enter a port: ");
    io::stdout().flush();
    stdin.read_line(&mut port);
    let port_int = port.trim().parse::<u32>().expect("could not make port into an int");
    let trimmed_host = host.trim().to_string();
    trimmed_host + &":".to_string() + &port_int.to_string()
}

pub fn initial_hello_msg() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::InitSetup,
        game_status: GameStatus::NotInGame,
        data: String::new()
    }
}

pub fn handle_out_of_game(connection: String, user_nick: String) -> Msg {
    loop {
        let mut stdin = io::stdin();
        let mut selection = String::new();
        print!("
    Now connected to: {0}.
    Your current nickname is: {1}.
    Welcome to Mancala.  Please select one of the following options:
        (1) Change Nickname
        (2) List Active Games
        (3) Start New Game
        (4) Join Game
        (5) Disconnect

    Enter your choice:
    ", connection, "nickname".to_string());
        io::stdout().flush();
        stdin.read_line(&mut selection);
        let selection_int = selection.trim().parse::<u8>();
        match selection_int {
            Ok(choice) => {
                match choice {
                    1 => {
                        println!("\n");
                        let msg = set_nickname();
                        return msg
                    }
                    2 => {
                        let msg = list_active_games();
                        return msg
                    }
                    3 => {
//                        let msg = start_new_game();
//                        return msg
                    }
                    4 => {
//                        let msg = join_game();
//                        return msg
                    }
                    5 => {
                        let msg = client_disconnect();
                        return msg
                    }
                    _ => {
                        println!("invalid selection");
                    }
                }
            }
            Err(e) => {
                error!("Could not read that input! More info: {}", e);
            }
        }
    }
}


// READ functions
pub fn list_active_games() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::ListGames,
        game_status: GameStatus::NotInGame,
        data: String::new()
    }
}

pub fn list_active_users() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::ListUsers,
        game_status: GameStatus::NotInGame,
        data: String::new()
    }
}

// pub fn get_game_info() -> Msg {}


// WRITE functions
pub fn set_nickname() -> Msg {
    let mut stdin = io::stdin();
    let mut nickname = String::new();
    print!("Enter new nickname: ");
    io::stdout().flush().expect("Client input something nonesensical");
    io::stdin().read_line(&mut nickname).expect("I/O error");
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::SetNick,
        game_status: GameStatus::NotInGame,
        data: nickname.trim_end().parse().unwrap()
    }
}

//pub fn start_new_game() -> Msg {}
//
//pub fn join_game() -> Msg {}

pub fn client_disconnect() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::KillMe,
        game_status: GameStatus::NotInGame,
        data: String::new()
    }
}


// --------------- in game --------------- //
pub fn handle_in_game() {}

// Response to Client
pub fn clients_turn() {}

pub fn send_game_state() {}

// Respond to Client's Actions
pub fn make_move() {}

pub fn leave_game() {} // TODO - implement?

pub fn send_message() {} // TODO - implement?