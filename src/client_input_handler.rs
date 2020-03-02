use crate::game_objects::*;
use crate::proto::*;
use std::io;
use std::io::prelude::*;
use std::ops::Range;
use std::{thread, time};

// --------------- out of game --------------- //

/// Initial input screen.  Asks the client for a host and port
/// to connect to.  Returns a connection string.
pub fn initial_screen() -> String {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut host = String::new();
    let mut port = String::new();
    print!("Enter a host: ");
    stdout.flush().expect("Error flushing buffer");
    stdin.read_line(&mut host).expect("Error reading in");
    print!("Enter a port: ");
    stdout.flush().expect("Error flushing buffer");
    stdin.read_line(&mut port).expect("Error reading in");
    let port_int = port
        .trim()
        .parse::<u32>()
        .expect("could not make port into an int");
    let trimmed_host = host.trim().to_string();
    trimmed_host + &":".to_string() + &port_int.to_string()
}

/// Returns the first message necessary for the client
pub fn initial_hello_msg() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::InitSetup,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty(),
    }
}

///
pub fn handle_out_of_game(connection: &str, user_nick: &str) -> Msg {
    loop {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut selection = String::new();
        print!(
            "
    Now connected to: {0}.
    Your current nickname is: {1}.
    Welcome to Mancala.  Please select one of the following options:
        (1) Change Nickname
        (2) List Active Games
        (3) List Active Users
        (4) Start New Game
        (5) Join Game
        (6) Disconnect

    Enter your choice: ",
            connection, user_nick
        );
        stdout.flush().expect("Error flushing buffer");
        stdin.read_line(&mut selection).expect("Error reading in");
        let selection_int = selection.trim().parse::<u8>();
        match selection_int {
            Ok(choice) => match choice {
                1 => {
                    println!("\n");
                    return set_nickname();
                }
                2 => {
                    return list_active_games();
                }
                3 => {
                    return list_active_users();
                }
                4 => {
                    return start_new_game();
                }
                5 => {
                    return join_game();
                }
                6 => {
                    return client_disconnect();
                }
                _ => {
                    println!("invalid selection");
                }
            },
            Err(e) => {
                error!("Could not read that input! More info: {}", e);
            }
        }
    }
}

// --------------- read functions --------------- //
fn list_active_games() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::ListGames,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty(),
    }
}

fn list_active_users() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::ListUsers,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty(),
    }
}

// --------------- write functions --------------- //

/// Creates a message, given a game id, asking the server
/// to add a player to a game
fn join_game() -> Msg {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut game_id = String::new();
    print!("Which Game Id do you want to join: ");
    stdout.flush().expect("Client input something nonsensical");
    stdin.read_line(&mut game_id).expect("I/O error");
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::JoinGame,
        game_status: GameStatus::NotInGame,
        data: game_id.trim().to_string(),
        game_state: GameState::new_empty(),
    }
}

/// Creates a message to ask the server to change the clients current nickname
fn set_nickname() -> Msg {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut nickname = String::new();
    print!("Enter new nickname: ");
    stdout.flush().expect("Client input something nonsensical");
    stdin.read_line(&mut nickname).expect("I/O error");
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::SetNick,
        game_status: GameStatus::NotInGame,
        data: nickname.trim().to_string(),
        game_state: GameState::new_empty(),
    }
}

pub fn start_new_game() -> Msg {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut game_name = String::new();
    print!("Enter a name of a new game: ");
    stdout.flush().expect("Error flushing buffer");
    stdin.read_line(&mut game_name).expect("Error reading in");
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::MakeNewGame,
        game_status: GameStatus::NotInGame,
        data: game_name.trim().to_string(),
        game_state: GameState::new_empty(),
    }
}

fn client_disconnect() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::KillMe,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty(),
    }
}

// --------------- in game --------------- //

/// Main functionality to handle client IO while in game.  Collections
/// moves while client's turn is active.
pub fn handle_in_game(server_msg: &Msg, my_id: u32) -> Msg {
    if server_msg.status != Status::Ok {
        return Msg {
            status: Status::Ok,
            headers: Headers::Read,
            command: Commands::KillMe,
            game_status: GameStatus::NotInGame,
            data: String::new(),
            game_state: GameState::new_empty(),
        };
    }
    if server_msg.command == Commands::GameIsOver {
        println!("Game Over!");
        render_board(server_msg);
        return leave_game();
    }
    if !server_msg.game_state.active {
        println!("Waiting for another player...");
        return wait_for_my_turn();
    }
    let am_i_player_one: bool = my_id == server_msg.game_state.player_one;
    println!("Current game state: ");
    render_board(server_msg);
    if (am_i_player_one && server_msg.game_state.player_one_turn)
        || (!am_i_player_one && !server_msg.game_state.player_one_turn)
    {
        make_move(am_i_player_one, &server_msg.game_state)
    } else {
        println!("Waiting for my turn...");
        wait_for_my_turn()
    }
}

// Response to Client
fn get_current_gamestate() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::GetCurrentGamestate,
        game_status: GameStatus::InGame,
        data: String::new(),
        game_state: GameState::new_empty(),
    }
}

fn wait_for_my_turn() -> Msg {
    let two_sec = time::Duration::from_secs(2);
    thread::sleep(two_sec);
    get_current_gamestate()
}

fn check_is_move_valid(move_to_make: usize, game_state: &GameState, am_i_player_one: bool) -> bool {
    let range_of_valid_moves = if am_i_player_one {
        Range {
            start: 1,
            end: SLOTS,
        }
    } else {
        Range {
            start: SLOTS + 1,
            end: BOARD_LENGTH,
        }
    };
    if range_of_valid_moves.contains(&move_to_make) && game_state.game_board[move_to_make] != 0 {
        return true;
    }
    if !range_of_valid_moves.contains(&move_to_make) {
        println!(
            "Move was not in the playable range of moves, which is {:?} for you.",
            &range_of_valid_moves
        );
    } else if game_state.game_board[move_to_make] == 0 {
        println!("slot you selected (slot {}) is empty!", &move_to_make);
    }
    false
}

fn make_move(am_i_player_one: bool, cur_game_state: &GameState) -> Msg {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let final_move: usize;
    loop {
        let mut move_to_make = String::new();
        print!("Which slot do you want to move: ");
        stdout.flush().expect("Error flushing buffer");
        stdin
            .read_line(&mut move_to_make)
            .expect("Error reading in");
        let move_to_make_int: usize;
        match move_to_make.trim().parse() {
            Ok(x) => {
                move_to_make_int = x;
            }
            Err(e) => {
                println!("invalid integer + {}!", e);
                continue;
            }
        }
        if check_is_move_valid(move_to_make_int, cur_game_state, am_i_player_one) {
            final_move = move_to_make_int;
            break;
        }
    }
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::MakeMove,
        game_status: GameStatus::InGame,
        data: final_move.to_string(),
        game_state: GameState::new_empty(),
    }
}

fn render_board(msg: &Msg) {
    println!("{:?}", msg.game_state.get_board());
    println!(
        "Player One score: {}",
        msg.game_state.get_player_one_score()
    );
    println!(
        "Player Two score: {}",
        msg.game_state.get_player_two_score()
    );
}

fn leave_game() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::LeaveGame,
        game_status: GameStatus::InGame,
        data: String::new(),
        game_state: GameState::new_empty(),
    }
}

// --------------- handle server response --------------- //
pub fn handle_server_response(
    server_msg: &Msg,
    connection: &str,
    nickname: &mut String,
    my_id: u32,
) -> Msg {
    if !server_msg.data.is_empty() {
        println!("server response: {}", server_msg.data);
    }
    if server_msg.command == Commands::SetNick && server_msg.status == Status::Ok {
        let new_nick: String = server_msg.data.clone();
        *nickname = new_nick;
    }
    match server_msg.game_status {
        GameStatus::NotInGame => handle_out_of_game(connection, &nickname),
        _ => handle_in_game(server_msg, my_id),
    }
}
