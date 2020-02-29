use crate::proto::*;
use crate::server_input_handler::*;

use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub type MsgChanSender = mpsc::Sender<(u32, Msg)>;
pub type MsgChanReceiver = mpsc::Receiver<(u32, Msg)>;

fn handle_client_input(buffer: &[u8; 512], size: usize) -> Msg {
    let client_msg: Msg = bincode::deserialize(&buffer[0..size]).unwrap();
    info!("TCP data received: {:?}", client_msg);
    if client_msg.status != Status::Ok {
        // TODO - some sort of error checking
        ()
    }
    client_msg
}

fn handle_each_client(
    mut stream: TcpStream,
    snd_channel: &Arc<Mutex<MsgChanSender>>,
    rec_channel: &MsgChanReceiver,
    user_id: u32,
) {
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(size) => {
                if size == 0 {
                    info!("Client terminated connection");
                    break;
                }
                let msg_to_send_to_manager: Msg =
                    handle_client_input(&buffer, size);
                snd_channel
                    .lock()
                    .unwrap()
                    .send((user_id, msg_to_send_to_manager))
                    .unwrap();
                let response_from_manager: (u32, Msg) =
                    rec_channel.recv().expect("something wrong");
                response_from_manager.1.serialize(&mut buffer);
                stream
                    .write_all(&buffer)
                    .unwrap();
                stream.flush().unwrap();
            }
            Err(_) => {
                error!(
                    "An error occurred, terminating connection with {}",
                    stream.peer_addr().unwrap()
                );
                stream.shutdown(Shutdown::Both).unwrap();
            }
        }
    }
}

fn data_manager(
    cli_comms: Arc<Mutex<HashMap<u32, MsgChanSender>>>,
    rec_server_master: MsgChanReceiver,
    game_list_mutex: Arc<Mutex<Vec<String>>>,
    active_nicks_mutex: Arc<Mutex<HashSet<String>>>,
    id_nick_map_mutex: Arc<Mutex<HashMap<u32, String>>>
) {
    loop {
        let rec: (u32, Msg) = rec_server_master
            .recv()
            .expect("didn't get a message or something");
        let cli_com_base = cli_comms.lock().unwrap();
        let res_comm_channel = cli_com_base.get(&rec.0).expect("no id match");
        let status: GameStatus = rec.1.game_status.clone();
        let cmd: Commands = rec.1.command.clone();
        if status == GameStatus::NotInGame {
            let server_res: Msg = handle_out_of_game(
                cmd,
                &game_list_mutex,
                &active_nicks_mutex,
                &id_nick_map_mutex,
                &rec.1,
            rec.0);
            res_comm_channel.send((rec.0, server_res) ).expect("Error sending to thread");
            continue;
        }
        // else do in-game
    }
}

fn set_up_new_client(
    client_comms_mutex: &Arc<Mutex<HashMap<u32, MsgChanSender>>>,
    client_to_server_sender: &Arc<Mutex<MsgChanSender>>,
    cur_id: u32,
    active_nicks_mutex: &Arc<Mutex<HashSet<String>>>,
    id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>
) -> (Arc<Mutex<MsgChanSender>>, MsgChanReceiver) {
    let (send_server, rec_channel): (MsgChanSender, MsgChanReceiver) = mpsc::channel();
    client_comms_mutex
        .lock()
        .unwrap()
        .insert(cur_id, send_server);
    let initial_nick: String = "user_".to_string() + &cur_id.to_string();
    active_nicks_mutex
        .lock()
        .unwrap()
        .insert(initial_nick.clone());
    id_nick_map_mutex
        .lock()
        .unwrap()
        .insert(cur_id, initial_nick);
    let snd_channel = Arc::clone(&client_to_server_sender);
    (snd_channel, rec_channel)
}

fn tcp_connection_manager(
    client_comms_mutex: Arc<Mutex<HashMap<u32, MsgChanSender>>>,
    client_to_server_sender: Arc<Mutex<MsgChanSender>>,
    active_nicks_mutex: Arc<Mutex<HashSet<String>>>,
    id_nick_map_mutex: Arc<Mutex<HashMap<u32, String>>>
) {
    let connection = "localhost:42069";
    let listener = TcpListener::bind(connection).unwrap();
    let mut cur_id: u32 = 0;

    info!("Server listening on port 42069");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("New connection: {}", stream.peer_addr().unwrap());
                let channels: (Arc<Mutex<MsgChanSender>>, MsgChanReceiver) =
                    set_up_new_client(
                        &client_comms_mutex,
                        &client_to_server_sender,
                        cur_id,
                        &active_nicks_mutex,
                        &id_nick_map_mutex);
                thread::spawn(move || {
                    handle_each_client(stream, &channels.0, &channels.1, cur_id);
                });
                cur_id += 1;
            }
            Err(e) => {
                error!("Error: {}", e);
            }
        }
    }
}

pub fn run_server() {
    let game_list: Vec<String> = vec![];
    let game_list_mutex = Arc::new(Mutex::new(game_list));

    let active_nicks: HashSet<String> = HashSet::new();
    let active_nicks_mutex = Arc::new(Mutex::new(active_nicks));

    let id_nick_map: HashMap<u32, String> = HashMap::new();
    let id_nick_map_mutex = Arc::new(Mutex::new(id_nick_map));

    let (send_client_master, rec_server_master): (MsgChanSender, MsgChanReceiver) = mpsc::channel();
    let client_to_server_sender = Arc::new(Mutex::new(send_client_master));

    let client_comms: HashMap<u32, MsgChanSender> = HashMap::new();
    let client_comms_mutex = Arc::new(Mutex::new(client_comms));

    let client_comms_mutex_tcp_manager_copy = Arc::clone(&client_comms_mutex);
    let client_comms_mutex_client_manager_copy = Arc::clone(&client_comms_mutex);
    let active_nicks_mutex_data_copy = Arc::clone(&active_nicks_mutex);
    let id_nick_map_mutex_data_copy = Arc::clone(&id_nick_map_mutex);
    let active_nicks_mutex_tcp_copy = Arc::clone(&active_nicks_mutex);
    let id_nick_map_mutex_tcp_copy = Arc::clone(&id_nick_map_mutex);

    thread::spawn(move || {
        data_manager(
            client_comms_mutex_client_manager_copy,
            rec_server_master,
            game_list_mutex,
            active_nicks_mutex_data_copy,
            id_nick_map_mutex_data_copy
        );
    });
    tcp_connection_manager(
        client_comms_mutex_tcp_manager_copy,
        client_to_server_sender,
        active_nicks_mutex_tcp_copy,
        id_nick_map_mutex_tcp_copy
    );
}
