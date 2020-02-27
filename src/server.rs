use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type MsgChanSender = mpsc::Sender<(u32, InternalMessage)>;
type MsgChanReceiver = mpsc::Receiver<(u32, InternalMessage)>;

#[derive(Debug, Clone, PartialEq)]
enum InternalMessageType {
    Read,
    Write,
    Response,
}

#[derive(Debug, Clone, PartialEq)]
struct InternalMessageBody {
    get_data: String,
    write_data: String,
    response_data: String,
}

#[derive(Debug, Clone, PartialEq)]
struct InternalMessage {
    type_of_msg: InternalMessageType,
    body: InternalMessageBody,
}

fn handle_client_input(buffer: &[u8; 512], id: u32, size: usize) -> InternalMessage {
    let input = str::from_utf8(&buffer[0..size]).unwrap().trim_end(); // evenutally deserialize a proto mmessage
    println!("TCP data received: {}", input);
    if input == "add_me" {
        InternalMessage {
            type_of_msg: InternalMessageType::Write,
            body: InternalMessageBody {
                get_data: String::new(),
                write_data: id.to_string(),
                response_data: String::new(),
            },
        }
    } else {
        InternalMessage {
            type_of_msg: InternalMessageType::Read,
            body: InternalMessageBody {
                get_data: "List".to_string(),
                write_data: String::new(),
                response_data: String::new(),
            },
        }
    }
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
                    println!("Client terminated connection");
                    break;
                }
                let msg_to_send_to_manager: InternalMessage =
                    handle_client_input(&buffer, user_id, size);
                snd_channel
                    .lock()
                    .unwrap()
                    .send((user_id, msg_to_send_to_manager))
                    .unwrap();
                let response_from_manager: (u32, InternalMessage) =
                    rec_channel.recv().expect("something wrong");
                stream
                    .write_all(response_from_manager.1.body.response_data.as_bytes())
                    .unwrap();
                stream.flush().unwrap();
            }
            Err(_) => {
                println!(
                    "An error occurred, terminating connection with {}",
                    stream.peer_addr().unwrap()
                );
                stream.shutdown(Shutdown::Both).unwrap();
            }
        }
    }
}

fn manager(
    cli_comms: &Arc<Mutex<HashMap<u32, MsgChanSender>>>,
    rec_server_master: MsgChanReceiver,
    user_list: Arc<Mutex<Vec<String>>>,
) {
    loop {
        let rec: (u32, InternalMessage) = rec_server_master
            .recv()
            .expect("didn't get a message or something");
        let cli_com_base = cli_comms.lock().unwrap();
        let res_comm_channel = cli_com_base.get(&rec.0).expect("no id match");
        if rec.1.type_of_msg == InternalMessageType::Read {
            println!("recieved some read message");
            let user_list_unlocked = user_list.lock().unwrap();
            let user_list_string = user_list_unlocked
                .iter()
                .fold(String::new(), |acc, x| acc + x + ", ");
            res_comm_channel
                .send((
                    rec.0,
                    InternalMessage {
                        type_of_msg: InternalMessageType::Response,
                        body: InternalMessageBody {
                            get_data: String::new(),
                            write_data: String::new(),
                            response_data: user_list_string + "\0",
                        },
                    },
                ))
                .expect("seomthing wrong");
        } else if rec.1.type_of_msg == InternalMessageType::Write {
            let mut user_list_unlocked = user_list.lock().unwrap();
            user_list_unlocked.push(rec.1.body.write_data.clone());
            let user_added_str = format!("User Successfully added: {}", rec.1.body.write_data);
            res_comm_channel
                .send((
                    rec.0,
                    InternalMessage {
                        type_of_msg: InternalMessageType::Response,
                        body: InternalMessageBody {
                            get_data: String::new(),
                            write_data: String::new(),
                            response_data: user_added_str + "\0",
                        },
                    },
                ))
                .expect("seomthing wrong");
        } else {
            res_comm_channel
                .send((
                    rec.0,
                    InternalMessage {
                        type_of_msg: InternalMessageType::Response,
                        body: InternalMessageBody {
                            get_data: String::new(),
                            write_data: String::new(),
                            response_data: "Command Not Recognized".to_string() + "\0",
                        },
                    },
                ))
                .expect("seomthing wrong");
        }
    }
}

fn make_initial_user_list(u: &mut Vec<String>) {
    u.push("logan".to_string());
    u.push("belen".to_string());
    u.push("megan".to_string());
    u.push("meatloaf".to_string());
    u.push("rooney".to_string());
}

pub fn run_server() {
    let mut user_list: Vec<String> = vec![];
    make_initial_user_list(&mut user_list);
    let mut cur_id: u32 = 0;
    let connection = "localhost:42069";
    let listener = TcpListener::bind(connection).unwrap();

    let client_comms: HashMap<u32, MsgChanSender> = HashMap::new();
    let (send_client_master, rec_server_master): (
        MsgChanSender,
        MsgChanReceiver,
    ) = mpsc::channel();

    let client_to_server_sender = Arc::new(Mutex::new(send_client_master));
    let client_comms_mutex = Arc::new(Mutex::new(client_comms));
    let user_list_mutex = Arc::new(Mutex::new(user_list));
    let client_comms_mutex_copy = Arc::clone(&client_comms_mutex);
    thread::spawn(move || {
        manager(&client_comms_mutex_copy, rec_server_master, user_list_mutex);
    });

    println!("Server listening on port 42069");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let (send_server, rec_channel): (
                    MsgChanSender,
                    MsgChanReceiver,
                ) = mpsc::channel();
                client_comms_mutex
                    .lock()
                    .unwrap()
                    .insert(cur_id, send_server);
                let snd_channel = Arc::clone(&client_to_server_sender);
                thread::spawn(move || {
                    handle_each_client(stream, &snd_channel, &rec_channel, cur_id);
                });
                cur_id += 1;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
