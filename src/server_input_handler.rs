use crate::proto::*;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};

// --------------- out of game --------------- //
pub fn handle_out_of_game(
    cmd: Commands,
    game_list_mutex: &Arc<Mutex<Vec<String>>>,
    active_nicks_mutex: &Arc<Mutex<HashSet<String>>>,
    id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>,
    client_msg: &Msg,
    client_id: u32
) -> Msg {
    if cmd == Commands::ListGames {
        return list_active_games(game_list_mutex);
    }
    else if cmd == Commands::ListUsers {
        return list_active_users(active_nicks_mutex);
    }
    else if cmd == Commands::SetNick {
        return set_nickname(active_nicks_mutex, id_nick_map_mutex, client_msg, client_id);
    }
    else if cmd == Commands::KillMe {
        return client_disconnect(active_nicks_mutex, id_nick_map_mutex, client_id);
    }
    Msg {
        status: Status::NotOk,
        headers: Headers::Response,
        command: Commands::Reply,
        game_status: GameStatus::NotInGame,
        data: String::new()
    }
}


// READ functions
pub fn list_active_games(game_list_mutex: &Arc<Mutex<Vec<String>>>) -> Msg {
    let game_list_unlocked = game_list_mutex.lock().unwrap();
    let game_list_string: String = game_list_unlocked
        .iter()
        .fold("Available Games: \n".to_string(), |acc, x| acc + x);
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::Reply,
        game_status: GameStatus::NotInGame,
        data: game_list_string
    }
}

pub fn list_active_users(active_nicks_mutex: &Arc<Mutex<HashSet<String>>>) -> Msg {
    let active_nicks_unlocked = active_nicks_mutex.lock().unwrap();
    let active_nicks_string: String = active_nicks_unlocked
        .iter()
        .fold("Active Users: \n".to_string(), |acc, x| acc + x + "\n ");
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::Reply,
        game_status: GameStatus::NotInGame,
        data: active_nicks_string
    }
}

// pub fn get_game_info() -> Msg {}


// WRITE functions
pub fn set_nickname(
    active_nicks_mutex: &Arc<Mutex<HashSet<String>>>,
    id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>,
    client_msg: &Msg,
    client_id: u32) -> Msg {
    let nickname = client_msg.data.clone();
    let mut active_nicks_unlocked = active_nicks_mutex.lock().unwrap();
    if active_nicks_unlocked.contains(&nickname) {
        return Msg {
            status: Status::NotOk,
            headers: Headers::Response,
            command: Commands::Reply,
            game_status: GameStatus::NotInGame,
            data: "nickname already in use".to_string()
        };
    } else {
        let mut id_nick_map_unlocked = id_nick_map_mutex.lock().unwrap();
        id_nick_map_unlocked.insert(client_id, nickname.clone());
        active_nicks_unlocked.insert(nickname.clone());
        Msg {
            status: Status::Ok,
            headers: Headers::Response,
            command: Commands::Reply,
            game_status: GameStatus::NotInGame,
            data: format!("nickname: {} set", nickname.clone())
        }
    }
}

//pub fn start_new_game() -> Msg {}
//
//pub fn join_game() -> Msg {}

pub fn client_disconnect(
    active_nicks_mutex: &Arc<Mutex<HashSet<String>>>,
    id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>,
    client_id: u32
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
        data: format!("Nick {} successfully booted", nickname)
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