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
fn initial_setup(id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>, client_id: u32) -> Msg {
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

fn list_active_games(game_list_mutex: &Arc<Mutex<Vec<GameState>>>) -> Msg {
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

fn list_active_users(active_nicks_mutex: &Arc<Mutex<HashSet<String>>>) -> Msg {
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
fn set_nickname(
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


#[test]
fn test_start_new_game() {
    let game_list: Vec<GameState> = vec![];
    let game_list_m: Arc<Mutex<Vec<GameState>>> = Arc::new(Mutex::new(game_list));
    let id_game_map: HashMap<u32, u32> = HashMap::new();
    let id_game_map_m: Arc<Mutex<HashMap<u32, u32>>> = Arc::new(Mutex::new(id_game_map));
    let client_id: u32 = 10;
    let res_msg = start_new_game(
        &game_list_m,
        &id_game_map_m,
        client_id,
        "none".to_string()
    );
    assert_eq!(res_msg.status, Status::Ok);
    assert_eq!(res_msg.headers, Headers::Response);
    assert_eq!(res_msg.command, Commands::MakeNewGame);
    assert_eq!(res_msg.game_status, GameStatus::InGame);
    assert_eq!(res_msg.data, "New Game".to_string());
    assert_eq!(
        res_msg.game_state,
        *game_list_m.lock().unwrap().get(0).unwrap()
    );
    assert!(id_game_map_m.lock().unwrap().contains_key(&client_id));
    assert_eq!(
        *(id_game_map_m.lock().unwrap().get(&client_id).unwrap()),
        0
    );
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
    if game_list_unlocked.len() == 0 {
        Msg {
            status: Status::Ok,
            headers: Headers::Response,
            command: Commands::JoinGame,
            game_status: GameStatus::NotInGame,
            data: "No active games available. Please start a new game to begin.".to_string(),
            game_state: GameState::new_empty(),
        }
    } else if game_id > game_list_unlocked.len() - 1 {
        Msg {
            status: Status::Ok,
            headers: Headers::Response,
            command: Commands::JoinGame,
            game_status: GameStatus::NotInGame,
            data: "Invalid game id entered. View active game list for valid game id's".to_string(),
            game_state: GameState::new_empty(),
        }
    } else {
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
}

#[test]
fn test_join_game() {
    let game_list: Vec<GameState> = vec![];
    let game_list_m: Arc<Mutex<Vec<GameState>>> = Arc::new(Mutex::new(game_list));
    let id_game_map: HashMap<u32, u32> = HashMap::new();
    let id_game_map_m: Arc<Mutex<HashMap<u32, u32>>> = Arc::new(Mutex::new(id_game_map));
    let cli_msg = Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::JoinGame,
        game_status: GameStatus::NotInGame,
        data: "0".to_string(),
        game_state: GameState::new_empty(),
    };
    let client_id: u32 = 10;
    let player_one_id: u32 = 5;
    let game_id: u32 = 0;
    let new_game = GameState::new(player_one_id, String::new(), game_id);
    game_list_m
        .lock()
        .unwrap()
        .insert(game_id as usize, new_game);
    id_game_map_m.lock().unwrap().insert(player_one_id, game_id);
    let res_msg = join_game(&game_list_m, &id_game_map_m, client_id, &cli_msg);
    assert_eq!(res_msg.status, Status::Ok);
    assert_eq!(res_msg.headers, Headers::Response);
    assert_eq!(res_msg.command, Commands::JoinGame);
    assert_eq!(res_msg.data, "Joined Game ".to_string());
    assert_eq!(
        res_msg.game_state,
        *game_list_m.lock().unwrap().get(game_id as usize).unwrap()
    );
    assert!(id_game_map_m.lock().unwrap().contains_key(&player_one_id));
    assert!(id_game_map_m.lock().unwrap().contains_key(&client_id));
    assert_eq!(
        *(id_game_map_m.lock().unwrap().get(&player_one_id).unwrap()),
        game_id
    );
    assert_eq!(
        *(id_game_map_m.lock().unwrap().get(&client_id).unwrap()),
        game_id
    );
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

#[test]
fn test_client_disconnect() {
    let active_nicks: HashSet<String> = HashSet::new();
    let active_nicks_m: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(active_nicks));
    let id_nick_map: HashMap<u32, String> = HashMap::new();
    let id_nick_map_m: Arc<Mutex<HashMap<u32, String>>> = Arc::new(Mutex::new(id_nick_map));
    let nick: String = "asdf".to_string();
    let client_id: u32 = 10;
    active_nicks_m.lock().unwrap().insert(nick.clone());
    id_nick_map_m
        .lock()
        .unwrap()
        .insert(client_id, nick.clone());
    let res_msg = client_disconnect(&active_nicks_m, &id_nick_map_m, client_id);
    assert_eq!(res_msg.status, Status::Ok);
    assert_eq!(res_msg.headers, Headers::Response);
    assert_eq!(res_msg.command, Commands::KillClient);
    assert_eq!(res_msg.data, format!("Nick {} successfully booted", nick));
    assert!(!active_nicks_m.lock().unwrap().contains(&nick));
    assert!(!id_nick_map_m.lock().unwrap().contains_key(&client_id));
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
        return make_move(client_msg, game, client_id);
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

#[test]
fn test_current_state_message() {
    let game = GameState::new(1, "asdf".to_string(), 1);
    let ret_msg = current_state(&game);
    assert_eq!(ret_msg.status, Status::Ok);
    assert_eq!(ret_msg.headers, Headers::Read);
    assert_eq!(ret_msg.command, Commands::Reply);
    assert_eq!(ret_msg.game_status, GameStatus::InGame);
    assert_eq!(ret_msg.game_state, game);
    assert_eq!(ret_msg.data, "Current Game State".to_string());
}

fn make_move(client_msg: &Msg, game: &mut GameState, client_id: u32) -> Msg {
    let move_to_make: u32 = client_msg.data.parse().unwrap();
    game.make_move(move_to_make as usize);
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::Reply,
        game_status: GameStatus::InGame,
        data: format!(
            "Player Id {} made move from slot {}",
            &client_id, &move_to_make
        ),
        game_state: game.clone(),
    }
}

#[test]
fn test_make_move_returns_message() {
    let mut game = GameState::new(1, "asdf".to_string(), 1);
    let cli_id: u32 = 1;
    let cli_msg = Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::MakeMove,
        game_status: GameStatus::InGame,
        data: "4".to_string(),
        game_state: game.clone(),
    };
    let ret_msg = make_move(&cli_msg, &mut game, cli_id);
    assert_eq!(ret_msg.status, Status::Ok);
    assert_eq!(ret_msg.headers, Headers::Read);
    assert_eq!(ret_msg.command, Commands::Reply);
    assert_eq!(ret_msg.game_status, GameStatus::InGame);
    assert_eq!(
        ret_msg.data,
        "Player Id 1 made move from slot 4".to_string()
    );
}
