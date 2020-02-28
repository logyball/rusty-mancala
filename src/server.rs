use crate::proto::*;

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
        // TODO - separate into read/write
        let cmd: Commands = rec.1.command;
        if cmd == Commands::ListGames {
            let game_list_unlocked = game_list_mutex.lock().unwrap();
            let game_list_string: String = game_list_unlocked
                .iter()
                .fold("Available Games: \n".to_string(), |acc, x| acc + x);
            let server_res: Msg = Msg {
                status: Status::Ok,
                headers: Headers::Response,
                command: Commands::Reply,
                data: game_list_string
            };
            res_comm_channel.send((rec.0, server_res) ).expect("Error sending to thread");
        }
        else if cmd == Commands::ListUsers {
            let active_nicks_unlocked = active_nicks_mutex.lock().unwrap();
            let active_nicks_string: String = active_nicks_unlocked
                .iter()
                .fold("Active Users: \n".to_string(), |acc, x| acc + x + "\n ");
            let server_res: Msg = Msg {
                status: Status::Ok,
                headers: Headers::Response,
                command: Commands::Reply,
                data: active_nicks_string
            };
            res_comm_channel.send((rec.0, server_res) ).expect("Error sending to thread");
        }
        else if cmd == Commands::SetNick {
            let nickname = rec.1.data;
            let mut active_nicks_unlocked = active_nicks_mutex.lock().unwrap();
            if active_nicks_unlocked.contains(&nickname) {
                let server_res: Msg = Msg {
                    status: Status::NotOk,
                    headers: Headers::Response,
                    command: Commands::Reply,
                    data: "nickname already in use".to_string()
                };
                res_comm_channel.send((rec.0, server_res) ).expect("Error sending to thread");
            } else {
                let mut id_nick_map_unlocked = id_nick_map_mutex.lock().unwrap();
                id_nick_map_unlocked.insert(rec.0, nickname.clone());
                active_nicks_unlocked.insert(nickname.clone());
                let server_res: Msg = Msg {
                    status: Status::Ok,
                    headers: Headers::Response,
                    command: Commands::Reply,
                    data: format!("nickname: {} set", nickname.clone())
                };
                res_comm_channel.send((rec.0, server_res) ).expect("Error sending to thread");
            }
        }
        else {
            let server_res: Msg = Msg {
                status: Status::NotOk,
                headers: Headers::Response,
                command: Commands::Reply,
                data: String::new()
            };
            res_comm_channel.send((rec.0, server_res) ).expect("Error sending to thread");
        }
    }
}

fn set_up_new_client(
    client_comms_mutex: &Arc<Mutex<HashMap<u32, MsgChanSender>>>,
    client_to_server_sender: &Arc<Mutex<MsgChanSender>>,
    cur_id: u32,
) -> (Arc<Mutex<MsgChanSender>>, MsgChanReceiver) {
    let (send_server, rec_channel): (MsgChanSender, MsgChanReceiver) = mpsc::channel();
    client_comms_mutex
        .lock()
        .unwrap()
        .insert(cur_id, send_server);
    let snd_channel = Arc::clone(&client_to_server_sender);
    (snd_channel, rec_channel)
}

fn tcp_connection_manager(
    client_comms_mutex: Arc<Mutex<HashMap<u32, MsgChanSender>>>,
    client_to_server_sender: Arc<Mutex<MsgChanSender>>,
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
                    set_up_new_client(&client_comms_mutex, &client_to_server_sender, cur_id);
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

    let client_comms_mute_tcp_manager_copy = Arc::clone(&client_comms_mutex);
    let client_comms_mute_client_manager_copy = Arc::clone(&client_comms_mutex);
    thread::spawn(move || {
        data_manager(
            client_comms_mute_client_manager_copy,
            rec_server_master,
            game_list_mutex,
            active_nicks_mutex,
            id_nick_map_mutex
        );
    });
    tcp_connection_manager(client_comms_mute_tcp_manager_copy, client_to_server_sender)
}
