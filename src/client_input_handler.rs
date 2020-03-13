use crate::constants::*;
use crate::game_objects::*;
use crate::proto::*;
use std::io;
use std::io::prelude::*;
use std::ops::Range;
use std::{thread, time};

// --------------- out of game --------------- //

/// Generates connection string based on the user's inputs for hostname
/// and port number. Used by the client to establish a connection to the server.
pub fn get_connection(host: String, port: u32) -> String {
    host.trim().to_string() + &":".to_string() + &port.to_string()
}

#[test]
fn test_get_connection() {
    let host: String = String::from("localhost");
    let port: u32 = 1234;
    let connection: String = String::from("localhost:1234");
    assert_eq!(get_connection(host, port), connection);
}

/// Collect the "host" field of the connection string from client
#[cfg_attr(tarpaulin, skip)]
pub fn get_host_input() -> String {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut host = String::new();
    loop {
        print!("Enter a host: ");
        stdout.flush().expect("Error flushing buffer");
        stdin.read_line(&mut host).expect("Error reading in");
        if !verify_host(host.trim().to_string()) {
            println!("Invalid host entered!");
            host = String::new();
            continue;
        }
        break;
    }
    host
}

fn verify_host(hostname: String) -> bool {
    !hostname.contains(' ') && !hostname.is_empty() && !hostname.contains(':')
}

#[test]
fn test_verify_invalid_host() {
    let mut invalid_host = String::from("");
    assert!(!verify_host(invalid_host));
    invalid_host = String::from("with space");
    assert!(!verify_host(invalid_host));
    invalid_host = String::from("with:colon");
    assert!(!verify_host(invalid_host));
}

#[test]
fn test_verify_valid_host() {
    let mut valid_host = String::from("localhost");
    assert!(verify_host(valid_host));
    valid_host = String::from("0.0.0.0");
    assert!(verify_host(valid_host));
    valid_host = String::from("ec2-52-11-55-180.us-west-2.compute.amazonaws.com");
    assert!(verify_host(valid_host));
}

/// Collect the "port" field of the connection string from client
#[cfg_attr(tarpaulin, skip)]
pub fn get_port_input() -> u32 {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut port = String::new();
    loop {
        print!("Enter a port: ");
        stdout.flush().expect("Error flushing buffer");
        stdin.read_line(&mut port).expect("Error reading in");

        if !verify_port(&port) {
            println!("Invalid port entered!");
            port = String::new();
            continue;
        }
        break;
    }
    port.trim().parse::<u32>().unwrap()
}

fn verify_port(port: &str) -> bool {
    match port.trim().parse::<u32>() {
        Ok(_) => true,
        Err(e) => {
            println!("could not make port into an int: {}!", e);
            false
        }
    }
}

#[test]
fn test_verify_invalid_port() {
    let invalid_port = String::from("12k");
    assert!(!verify_port(&invalid_port));
}

#[test]
fn test_verify_valid_port() {
    let valid_port = String::from("1234");
    assert!(verify_port(&valid_port));
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

#[test]
fn test_initial_hello_msg() {
    let valid_hello_msg = Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::InitSetup,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty(),
    };
    assert_eq!(initial_hello_msg(), valid_hello_msg);
}

/// Handle when a user is out of a game
#[cfg_attr(tarpaulin, skip)]
pub fn get_out_of_game_selection(connection: &str, user_nick: &str) -> u8 {
    let stdin = io::stdin();
    let mut selection = String::new();

    loop {
        display_menu(connection, user_nick);
        stdin.read_line(&mut selection).expect("Error reading in");

        if !verify_selection(&selection) {
            println!("Invalid menu selection entered!");
            selection = String::new();
            continue;
        }
        break;
    }
    selection.trim().parse::<u8>().unwrap()
}

fn verify_selection(selection: &str) -> bool {
    match selection.trim().parse::<u32>() {
        Ok(choice) => {
            if choice > 0 && choice < 7 {
                return true;
            }
            false
        }
        Err(e) => {
            println!("could not make selection into an int: {}!", e);
            false
        }
    }
}

#[test]
fn test_verify_invalid_selection() {
    let mut invalid_selection = String::from("invalid");
    assert!(!verify_selection(&invalid_selection));
    invalid_selection = String::from("8");
    assert!(!verify_selection(&invalid_selection));
}

#[test]
fn test_verify_valid_selection() {
    let mut valid_selection = String::from("6");
    assert!(verify_selection(&valid_selection));
    valid_selection = String::from("1");
    assert!(verify_selection(&valid_selection));
}

pub fn handle_out_of_game(selection: u8) -> Msg {
    match selection {
        1 => set_nickname(get_nickname_input()),
        2 => list_available_games(),
        3 => list_active_users(),
        4 => start_new_game(),
        5 => join_game(get_join_game_input()),
        _ => client_initiate_disconnect(),
    }
}

#[test]
fn test_handle_out_of_game() {
    let mut selection = handle_out_of_game(2);
    assert_eq!(selection, list_available_games());
    selection = handle_out_of_game(3);
    assert_eq!(selection, list_active_users());
}

#[cfg_attr(tarpaulin, skip)]
fn display_menu(connection: &str, user_nick: &str) {
    let mut stdout = io::stdout();
    print!(
        "
        Now connected to: {0}.
        Your current nickname is: {1}.
        Welcome to Mancala.  Please select one of the following options:
            (1) Change Nickname
            (2) List Available Games
            (3) List Active Users
            (4) Start New Game
            (5) Join Game
            (6) Disconnect

        Enter your choice: ",
        connection, user_nick
    );
    stdout.flush().expect("Error flushing buffer");
}

// --------------- read functions --------------- //

fn list_available_games() -> Msg {
    print!("{}[2J", 27 as char);
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::ListGames,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty(),
    }
}

#[test]
fn test_list_available_games() {
    let available_games = list_available_games();
    assert_eq!(available_games.status, Status::Ok);
    assert_eq!(available_games.headers, Headers::Read);
    assert_eq!(available_games.command, Commands::ListGames);
    assert_eq!(available_games.game_status, GameStatus::NotInGame);
    assert_eq!(available_games.data, String::new());
    assert_eq!(available_games.game_state, GameState::new_empty());
}

fn list_active_users() -> Msg {
    print!("{}[2J", 27 as char);
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::ListUsers,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty(),
    }
}

#[test]
fn test_list_active_users() {
    let active_users = list_active_users();
    assert_eq!(active_users.status, Status::Ok);
    assert_eq!(active_users.headers, Headers::Read);
    assert_eq!(active_users.command, Commands::ListUsers);
    assert_eq!(active_users.game_status, GameStatus::NotInGame);
    assert_eq!(active_users.data, String::new());
    assert_eq!(active_users.game_state, GameState::new_empty());
}

// --------------- write functions --------------- //

#[cfg_attr(tarpaulin, skip)]
fn get_join_game_input() -> usize {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut game_id = String::new();
    loop {
        print!("Which Game ID do you want to join: ");
        stdout.flush().expect("Client input something nonsensical");
        stdin.read_line(&mut game_id).expect("I/O error");

        if !verify_join_game(&game_id) {
            println!("Invalid game id entered!");
            game_id = String::new();
            continue;
        }
        break;
    }
    game_id.trim().parse::<usize>().unwrap()
}

fn verify_join_game(game_id: &str) -> bool {
    match game_id.trim().parse::<u32>() {
        Ok(_) => true,
        Err(e) => {
            println!("could not make game id into an int: {}!", e);
            false
        }
    }
}

#[test]
fn test_verify_invalid_join_game() {
    let invalid_game = String::from("12k");
    assert!(!verify_join_game(&invalid_game));
}

#[test]
fn test_verify_valid_join_game() {
    let valid_game = String::from("1234");
    assert!(verify_join_game(&valid_game));
}

/// Creates a message, given a game id, asking the server
/// to add a player to a game
fn join_game(game_id: usize) -> Msg {
    print!("{}[2J", 27 as char);
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::JoinGame,
        game_status: GameStatus::NotInGame,
        data: game_id.to_string(),
        game_state: GameState::new_empty(),
    }
}

#[test]
fn test_join_game() {
    let game_id: usize = 0;
    let join_game_msg = join_game(game_id);
    assert_eq!(join_game_msg.status, Status::Ok);
    assert_eq!(join_game_msg.headers, Headers::Write);
    assert_eq!(join_game_msg.command, Commands::JoinGame);
    assert_eq!(join_game_msg.game_status, GameStatus::NotInGame);
    assert_eq!(join_game_msg.data, game_id.to_string());
    assert_eq!(join_game_msg.game_state, GameState::new_empty());
}

fn get_nickname_input() -> String {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut nickname = String::new();
    println!("\n");
    print!("Enter new nickname: ");
    stdout.flush().expect("Client input something nonsensical");
    stdin.read_line(&mut nickname).expect("I/O error");
    nickname
}

/// Creates a message to ask the server to change the clients current nickname
fn set_nickname(nickname: String) -> Msg {
    print!("{}[2J", 27 as char);
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::SetNick,
        game_status: GameStatus::NotInGame,
        data: nickname.trim().to_string(),
        game_state: GameState::new_empty(),
    }
}

#[test]
fn test_set_nickname() {
    let nickname: String = String::from("rooney");
    let set_nickname_msg = set_nickname(nickname);
    assert_eq!(set_nickname_msg.status, Status::Ok);
    assert_eq!(set_nickname_msg.headers, Headers::Write);
    assert_eq!(set_nickname_msg.command, Commands::SetNick);
    assert_eq!(set_nickname_msg.game_status, GameStatus::NotInGame);
    assert_eq!(set_nickname_msg.data, String::from("rooney"));
    assert_eq!(set_nickname_msg.game_state, GameState::new_empty());
}

/// Creates a message to ask the server to start a new game as well as add the client to the game
pub fn start_new_game() -> Msg {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut game_name = String::new();
    print!("Enter a name of a new game: ");
    stdout.flush().expect("Error flushing buffer");
    stdin.read_line(&mut game_name).expect("Error reading in");
    print!("{}[2J", 27 as char);
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::MakeNewGame,
        game_status: GameStatus::NotInGame,
        data: game_name.trim().to_string(),
        game_state: GameState::new_empty(),
    }
}

pub fn client_initiate_disconnect() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::KillMe,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty(),
    }
}

#[test]
fn test_client_init_disconnect() {
    let client_disconnect = client_initiate_disconnect();
    assert_eq!(client_disconnect.status, Status::Ok);
    assert_eq!(client_disconnect.headers, Headers::Read);
    assert_eq!(client_disconnect.command, Commands::KillMe);
    assert_eq!(client_disconnect.game_status, GameStatus::NotInGame);
    assert_eq!(client_disconnect.data, String::new());
    assert_eq!(client_disconnect.game_state, GameState::new_empty());
}

// --------------- in game --------------- //

/// Main functionality to handle client IO while in game.  Collects
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
        print!("{}[2J", 27 as char);
        println!("Game Over!");
        render_board(&server_msg.game_state.get_board(), am_i_player_one);
        return leave_game();
    }
    if !server_msg.game_state.active {
        print!("{}[2J", 27 as char);
        println!("Waiting for another player...");
        return wait_for_my_turn();
    }
    print!("{}[2J", 27 as char);
    println!("Current game state: ");
    render_board(&server_msg.game_state.get_board(), am_i_player_one);
    if (am_i_player_one && server_msg.game_state.player_one_turn)
        || (!am_i_player_one && !server_msg.game_state.player_one_turn)
    {
        make_move(am_i_player_one, &server_msg.game_state)
    } else {
        println!("\n\n\tWaiting for my turn...\n\n\n");
        wait_for_my_turn()
    }
}

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

#[test]
fn test_get_current_gamestate() {
    let current_gamestate = get_current_gamestate();
    assert_eq!(current_gamestate.status, Status::Ok);
    assert_eq!(current_gamestate.headers, Headers::Read);
    assert_eq!(current_gamestate.command, Commands::GetCurrentGamestate);
    assert_eq!(current_gamestate.game_status, GameStatus::InGame);
    assert_eq!(current_gamestate.data, String::new());
    assert_eq!(current_gamestate.game_state, GameState::new_empty());
}

fn wait_for_my_turn() -> Msg {
    let two_sec = time::Duration::from_secs(4);
    thread::sleep(two_sec);
    get_current_gamestate()
}

/// Helper function to decide if the current game state allows the requested move, otherwise
/// return an error
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

#[test]
fn test_invalid_move() {
    let move_to_make: usize = 6;
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    gs.add_new_player(2);
    let player_one = false;
    assert!(!check_is_move_valid(move_to_make, &gs, player_one));
}

#[test]
fn test_valid_move() {
    let move_to_make: usize = 6;
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    gs.add_new_player(2);
    let player_one = true;
    assert!(check_is_move_valid(move_to_make, &gs, player_one));
}

/// Loop to collect the slot to move from the player, validate it for a "legal" move,
/// then send off to the server for processing
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

#[cfg_attr(tarpaulin, skip)]
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

pub fn leave_game() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::LeaveGame,
        game_status: GameStatus::InGame,
        data: String::new(),
        game_state: GameState::new_empty(),
    }
}

#[test]
fn test_leave_game() {
    let leave_game_msg = leave_game();
    assert_eq!(leave_game_msg.status, Status::Ok);
    assert_eq!(leave_game_msg.headers, Headers::Write);
    assert_eq!(leave_game_msg.command, Commands::LeaveGame);
    assert_eq!(leave_game_msg.game_status, GameStatus::InGame);
    assert_eq!(leave_game_msg.data, String::new());
    assert_eq!(leave_game_msg.game_state, GameState::new_empty());
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
        GameStatus::NotInGame => {
            let selection = get_out_of_game_selection(connection, &nickname);
            handle_out_of_game(selection)
        }
        _ => handle_in_game(server_msg, my_id),
    }
}
