use crate::game_objects::*;
use crate::proto::*;
use std::io;
use std::io::prelude::*;
use std::ops::Range;
use std::{thread, time};

// --------------- out of game --------------- //

/// Initial input screen.  Calls helper functions to get valid host and port
/// to connect to.  Returns a connection string.
pub fn initial_screen() -> String {
    let host: String = get_host_input();
    let port_int: u32 = get_port_input();

    host.trim().to_string() + &":".to_string() + &port_int.to_string()
}

fn get_host_input() -> String {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut host = String::new();
    loop {
        print!("Enter a host: ");
        stdout.flush().expect("Error flushing buffer");
        stdin.read_line(&mut host).expect("Error reading in");
        if host.trim().is_empty() {
            println!("Cannot have an empty host!");
            continue;
        }
        break;
    }
    host
}

fn get_port_input() -> u32 {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let port_int: u32;
    loop {
        let mut port = String::new();
        print!("Enter a port: ");
        stdout.flush().expect("Error flushing buffer");
        stdin.read_line(&mut port).expect("Error reading in");
        match port.trim().parse() {
            Ok(x) => {
                port_int = x;
                break;
            }
            Err(e) => {
                println!("could not make port into an int: {}!", e);
                continue;
            }
        }
    }
    port_int
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
    let game_id: usize;
    loop {
        let mut game_id_input = String::new();
        print!("Which Game ID do you want to join: ");
        stdout.flush().expect("Client input something nonsensical");
        stdin.read_line(&mut game_id_input).expect("I/O error");
        match game_id_input.trim().parse() {
            Ok(x) => {
                game_id = x;
                break;
            }
            Err(e) => {
                println!("invalid integer + {}!", e);
                continue;
            }
        }
    }
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::JoinGame,
        game_status: GameStatus::NotInGame,
        data: game_id.to_string(),
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
    let am_i_player_one: bool = my_id == server_msg.game_state.player_one;
    if server_msg.command == Commands::GameIsOver {
        println!("Game Over!");
        render_board(&server_msg.game_state.get_board(), am_i_player_one);
        return leave_game();
    }
    if !server_msg.game_state.active {
        println!("Waiting for another player...");
        return wait_for_my_turn();
    }
    println!("Current game state: ");
    render_board(&server_msg.game_state.get_board(), am_i_player_one);
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
            "Move was not in the playable range of moves, which is {} -> {} for you.",
            &range_of_valid_moves.start,
            &range_of_valid_moves.end - 1
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
        if am_i_player_one {
            println!("Player 1, enter your move (1 - 6)");
        } else {
            println!("Player 2, enter your move (8 - 13)");
        }
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

fn render_board(board: &[u8; BOARD_LENGTH], am_i_player_one: bool) {
    let range_top = if am_i_player_one {
        Range {
            start: SLOTS + 2,
            end: BOARD_LENGTH,
        }
    } else {
        Range {
            start: 2,
            end: SLOTS,
        }
    };
    let range_bottom = if am_i_player_one {
        Range {
            start: 1,
            end: SLOTS - 1,
        }
    } else {
        Range {
            start: SLOTS + 1,
            end: BOARD_LENGTH - 1,
        }
    };
    let goal1 = if am_i_player_one { 0 } else { SLOTS };
    let goal2 = if am_i_player_one { SLOTS } else { 0 };
    // Player1
    print!("\t  ");
    for i in range_top.rev() {
        if i < 10 {
            print!("  #{}: {} |", i, board[i]);
        } else {
            print!(" #{}: {} |", i, board[i]);
        }
    }
    if am_i_player_one {
        println!(" #{}: {}", &SLOTS + 1, board[SLOTS + 1])
    } else {
        println!(" #{}: {}", &1, board[1])
    };
    // Scores
    print!("\t{} ", board[goal1]);
    for _i in 1..6 {
        print!("--------+")
    }
    println!("-------- {}", board[goal2]);
    print!("\t  ");
    for i in range_bottom {
        if i < 10 {
            print!("  #{}: {} |", i, board[i]);
        } else {
            print!(" #{}: {} |", i, board[i]);
        }
    }
    if am_i_player_one {
        println!(" #{}: {}", &SLOTS - 1, board[SLOTS - 1])
    } else {
        println!(" #{}: {}", &BOARD_LENGTH - 1, board[BOARD_LENGTH - 1])
    };
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
