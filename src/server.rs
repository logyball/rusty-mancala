use rand::Rng;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::str;
use std::thread;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, Shutdown};

//fn backup() {
//    let rec: (u32, String) = rec_server_master
//        .recv()
//        .expect("didn't get a message or something");
//    let res_comm_channel = client_comms.get(&rec.0).expect("no id match");
//    if rec.1 == "kill_me".to_string() {
//        res_comm_channel
//            .send((rec.0, "kill".to_string()))
//            .expect("something wrong");
//        client_comms.remove(&rec.0);
//    } else if rec.1 == "give".to_string() {
//        res_comm_channel
//            .send((rec.0, user_list[(rec.0 % 5) as usize].to_string()))
//            .expect("seomthing wrong");
//    } else if rec.1 == "list".to_string() {
//        let new_str = user_list
//            .iter()
//            .fold(String::new(), |acc, n| acc + n);
//        res_comm_channel
//            .send((rec.0, new_str))
//            .expect("seomthing wrong");
//    } else if rec.1 == "add_me".to_string() {
//        user_list.push(format!("new_user_{}", rec.0));
//        res_comm_channel
//            .send((rec.0, String::new()))
//            .expect("seomthing wrong");
//    }
//}

fn handle_each_client(
    mut stream: TcpStream,
    snd_channel: &Arc<Mutex<mpsc::Sender<(u32, String)>>>,
    rec_channel: &mpsc::Receiver<(u32, String)>,
    user_id: u32)
{
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(size) => {
                if size == 0 {
                    println!("Client terminated connection");
                    break;
                }
                let input = str::from_utf8(&buffer[0..size]).unwrap().trim_end();
                println!("TCP data received: {}", input);
                snd_channel.lock().unwrap().send((user_id, input.to_string())).unwrap();
                let msg: (u32, String) = rec_channel.recv().expect("something wrong");
                println!("internal data recieved: {}", msg.1);
                let terminated_msg = msg.1 + "\n";
                stream.write_all(terminated_msg.as_bytes()).unwrap();
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

fn make_user_list(u: &mut Vec<String>) {
    u.push("logan".to_string());
    u.push("belen".to_string());
    u.push("megan".to_string());
    u.push("meatloaf".to_string());
    u.push("rooney".to_string());
}

fn manager(
    cli_comms: &Arc<Mutex<HashMap<u32, mpsc::Sender<(u32, String)>>>>,
    rec_server_master: mpsc::Receiver<(u32, String)>,
    user_list: Arc<Mutex<Vec<String>>>
) {
    loop {
        let rec: (u32, String) = rec_server_master
            .recv()
            .expect("didn't get a message or something");
        let cli_com_base = cli_comms.lock().unwrap();
        let res_comm_channel = cli_com_base.get(&rec.0).expect("no id match");
        if rec.1 == "list".to_string() {
            let user_list_unlocked = user_list.lock().unwrap();
            let user_list_string = user_list_unlocked.iter().fold(String::new(), |acc, x| acc + x);
            res_comm_channel
                .send((rec.0, user_list_string))
                .expect("seomthing wrong");
        } else {
            res_comm_channel
                .send((rec.0, "command not recognized".to_string()))
                .expect("seomthing wrong");
        }
    }
}

pub fn run_server() {
    let mut user_list: Vec<String> = vec![]; make_user_list(&mut user_list);
    let mut cur_id: u32 = 0;
    let connection = "localhost:42069";
    let listener = TcpListener::bind(connection).unwrap();

    let mut client_comms: HashMap<u32, mpsc::Sender<(u32, String)>> = HashMap::new();
    let (send_client_master, rec_server_master): (
        mpsc::Sender<(u32, String)>,
        mpsc::Receiver<(u32, String)>,
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
                let (send_server, rec_channel): (mpsc::Sender<(u32, String)>, mpsc::Receiver<(u32, String)>) =
                    mpsc::channel();
                client_comms_mutex.lock().unwrap().insert(cur_id, send_server);
                let snd_channel = Arc::clone(&client_to_server_sender);
                thread::spawn(move || {
                    handle_each_client(stream,
                                       &snd_channel,
                                       &rec_channel,
                                       cur_id);
                });
                cur_id += 1;
            }
            Err(e) => { println!("Error: {}", e); }
        }
    }
}