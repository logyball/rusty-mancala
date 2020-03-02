use crate::game_objects::*;
use crate::proto::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

// --------------- out of game --------------- //

/// When the player is in the "lobby", they will send messages
/// to the server and this is where they are handled.
pub fn handle_out_of_game(
    cmd: Commands,
    game_list_mutex: &Arc<Mutex<Vec<GameState>>>,
    id_game_map_mutex: &Arc<Mutex<HashMap<u32, u32>>>,
    active_nicks_mutex: &Arc<Mutex<HashSet<String>>>,
    id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>,
    client_msg: &Msg,
    client_id: u32,
) -> Msg {
    match cmd {
        Commands::InitSetup => initial_setup(id_nick_map_mutex, client_id),
        Commands::ListGames => list_active_games(game_list_mutex),
        Commands::ListUsers => list_active_users(active_nicks_mutex),
        Commands::SetNick => {
            set_nickname(active_nicks_mutex, id_nick_map_mutex, client_msg, client_id)
        }
        Commands::KillMe => client_disconnect(active_nicks_mutex, id_nick_map_mutex, client_id),
        Commands::MakeNewGame => start_new_game(
            game_list_mutex,
            id_game_map_mutex,
            client_id,
            client_msg.data.clone(),
        ),
        Commands::JoinGame => join_game(game_list_mutex, id_game_map_mutex, client_id, client_msg),
        _ => Msg {
            status: Status::NotOk,
            headers: Headers::Response,
            command: Commands::Reply,
            game_status: GameStatus::NotInGame,
            data: String::new(),
            game_state: GameState::new_empty(),
        },
    }
}

// --------------- out of game READ functions --------------- //
pub fn initial_setup(id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>, client_id: u32) -> Msg {
    let id_nick_map_unlocked = id_nick_map_mutex.lock().unwrap();
    let nickname = id_nick_map_unlocked.get(&client_id).unwrap();
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::Reply,
        game_status: GameStatus::NotInGame,
        data: format!("{}^{}", nickname, client_id),
        game_state: GameState::new_empty(),
    }
}

pub fn list_active_games(game_list_mutex: &Arc<Mutex<Vec<GameState>>>) -> Msg {
    let game_list_unlocked = game_list_mutex.lock().unwrap();
    let game_list_string: String = game_list_unlocked
        .iter()
        .fold("Available Games: \n".to_string(), |acc, x| {
            acc + &x.game_id.to_string() + ": " + &x.game_name + "\n"
        });
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::Reply,
        game_status: GameStatus::NotInGame,
        data: game_list_string,
        game_state: GameState::new_empty(),
    }
}

pub fn list_active_users(active_nicks_mutex: &Arc<Mutex<HashSet<String>>>) -> Msg {
    let active_nicks_unlocked = active_nicks_mutex.lock().unwrap();
    let active_nicks_string: String = active_nicks_unlocked
        .iter()
        .fold("Active Users: \n".to_string(), |acc, x| acc + x + "\n");
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::Reply,
        game_status: GameStatus::NotInGame,
        data: active_nicks_string,
        game_state: GameState::new_empty(),
    }
}

// --------------- out of game READ functions --------------- //
/// Sets a clients nickname based on a passed-in string.  Compares across
/// already registered nicknames and doesn't allow duplicate values
pub fn set_nickname(
    active_nicks_mutex: &Arc<Mutex<HashSet<String>>>,
    id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>,
    client_msg: &Msg,
    client_id: u32,
) -> Msg {
    let nickname = client_msg.data.clone();
    let mut active_nicks_unlocked = active_nicks_mutex.lock().unwrap();
    if active_nicks_unlocked.contains(&nickname) {
        Msg {
            status: Status::NotOk,
            headers: Headers::Response,
            command: Commands::SetNick,
            game_status: GameStatus::NotInGame,
            data: "nickname already in use".to_string(),
            game_state: GameState::new_empty(),
        }
    } else {
        let mut id_nick_map_unlocked = id_nick_map_mutex.lock().unwrap();
        let old_nick = id_nick_map_unlocked.remove(&client_id).unwrap();
        active_nicks_unlocked.remove(&old_nick);
        id_nick_map_unlocked.insert(client_id, nickname.clone());
        active_nicks_unlocked.insert(nickname.clone());
        Msg {
            status: Status::Ok,
            headers: Headers::Response,
            command: Commands::SetNick,
            game_status: GameStatus::NotInGame,
            data: nickname,
            game_state: GameState::new_empty(),
        }
    }
}

fn start_new_game(
    game_list_mutex: &Arc<Mutex<Vec<GameState>>>,
    id_game_map_mutex: &Arc<Mutex<HashMap<u32, u32>>>,
    client_id: u32,
    mut game_name: String,
) -> Msg {
    let mut game_list_unlocked = game_list_mutex.lock().unwrap();
    let mut id_game_map_unlocked = id_game_map_mutex.lock().unwrap();
    let game_id = game_list_unlocked.len() as u32;
    if game_name.is_empty() {
        game_name = "New Game".to_string();
    }
    let new_game = GameState::new(client_id, game_name, game_id);
    game_list_unlocked.push(new_game.clone());
    id_game_map_unlocked.insert(client_id, game_id);
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::MakeNewGame,
        game_status: GameStatus::InGame,
        data: "New Game".to_string(),
        game_state: new_game,
    }
}

fn join_game(
    game_list_mutex: &Arc<Mutex<Vec<GameState>>>,
    id_game_map_mutex: &Arc<Mutex<HashMap<u32, u32>>>,
    client_id: u32,
    client_msg: &Msg,
) -> Msg {
    let mut game_list_unlocked = game_list_mutex.lock().unwrap();
    let mut id_game_map_unlocked = id_game_map_mutex.lock().unwrap();
    let game_id: usize = client_msg.data.parse().unwrap();
    let game: &mut GameState = &mut game_list_unlocked[game_id];
    game.add_player_two(client_id);
    id_game_map_unlocked.insert(client_id, game.game_id);
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::JoinGame,
        game_status: GameStatus::InGame,
        data: format!("Joined Game {}", &game.game_name),
        game_state: game.clone(),
    }
}

/// Remove a client from the list of active users and send the message
/// that the client should be killed
fn client_disconnect(
    active_nicks_mutex: &Arc<Mutex<HashSet<String>>>,
    id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>,
    client_id: u32,
) -> Msg {
    let mut active_nicks_unlocked = active_nicks_mutex.lock().unwrap();
    let mut id_nick_map_unlocked = id_nick_map_mutex.lock().unwrap();
    let nickname = id_nick_map_unlocked.remove(&client_id).unwrap();
    active_nicks_unlocked.remove(&nickname);
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::KillClient,
        game_status: GameStatus::NotInGame,
        data: format!("Nick {} successfully booted", nickname),
        game_state: GameState::new_empty(),
    }
}

// --------------- in game --------------- //
pub fn handle_in_game(
    cmd: Commands,
    game_list_mutex: &Arc<Mutex<Vec<GameState>>>,
    id_game_map_mutex: &Arc<Mutex<HashMap<u32, u32>>>,
    client_msg: &Msg,
    client_id: u32,
) -> Msg {
    let id_game_map_unlocked = id_game_map_mutex.lock().unwrap();
    let mut game_list_unlocked = game_list_mutex.lock().unwrap();
    let game_id = id_game_map_unlocked.get(&client_id).unwrap();
    let game: &mut GameState = &mut game_list_unlocked[*game_id as usize];
    if game.player_one != 0 && game.player_two != 0 && !game.active {
        return Msg {
            status: Status::Ok,
            headers: Headers::Write,
            command: Commands::GameIsOver,
            game_status: GameStatus::NotInGame,
            data: "Game Over".to_string(),
            game_state: game.clone(),
        };
    }
    if cmd == Commands::GetCurrentGamestate {
        return current_state(game);
    } else if cmd == Commands::MakeMove {
        return make_move(client_msg, game);
    }
    Msg {
        status: Status::NotOk,
        headers: Headers::Read,
        command: Commands::Reply,
        game_status: GameStatus::NotInGame,
        data: "Somethings wrong".to_string(),
        game_state: game.clone(),
    }
}

fn current_state(game: &GameState) -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::Reply,
        game_status: GameStatus::InGame,
        data: "Current Game State".to_string(),
        game_state: game.clone(),
    }
}

pub fn make_move(client_msg: &Msg, game: &mut GameState) -> Msg {
    let move_to_make: u32 = client_msg.data.parse().unwrap();
    game.make_move(move_to_make as usize);
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::Reply,
        game_status: GameStatus::InGame,
        data: "Current Game State".to_string(),
        game_state: game.clone(),
    }
}
