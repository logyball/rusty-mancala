use rand::Rng;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

fn do_message_actions(
    snd_channel: &Arc<Mutex<mpsc::Sender<(u32, &str)>>>,
    rec_channel: &mpsc::Receiver<(u32, &str)>,
    id: u32,
) {
    thread::sleep(Duration::from_secs(1));
    snd_channel.lock().unwrap().send((id, "give")).unwrap();
    let mut rng = rand::thread_rng();
    loop {
        let mut rec: (u32, &str) = rec_channel
            .recv()
            .expect("didn't get a message or something");
        if rec.1 == "kill" {
            return ();
        }
        let mut zero_to_four = rng.gen_range(0, 6);
        if zero_to_four != 5 {
            snd_channel.lock().unwrap().send((id, "give")).unwrap();
        } else {
            println!("killing: {}", id);
            snd_channel.lock().unwrap().send((id, "kill_me")).unwrap();
        }
        println!("client recieved: {} {:?}", id, rec.1);
    }
}

#[derive(Debug)]
struct Comms<'a> {
    id: u32,
    srv_send: mpsc::Sender<(u32, &'a str)>,
}

fn main() {
    let mut user_list: Vec<&str> = vec![];
    let mut client_comms: HashMap<u32, Comms> = HashMap::new();
    let (send_client_master, rec_server_master): (
        mpsc::Sender<(u32, &str)>,
        mpsc::Receiver<(u32, &str)>,
    ) = mpsc::channel();
    let snd_cli = Arc::new(Mutex::new(send_client_master));
    user_list.push("logan");
    user_list.push("belen");
    user_list.push("megan");
    user_list.push("meatloaf");
    for i in 0..10 {
        let client_id: u32 = i;
        let (send_server, rec_client): (mpsc::Sender<(u32, &str)>, mpsc::Receiver<(u32, &str)>) =
            mpsc::channel();
        let snd_channel = Arc::clone(&snd_cli);
        let child = thread::spawn(move || {
            do_message_actions(&snd_channel, &rec_client, client_id);
        });
        let cli_comms = Comms {
            id: client_id,
            srv_send: send_server,
        };
        client_comms.insert(client_id, cli_comms);
    }
    loop {
        if client_comms.is_empty() {
            break;
        }
        let mut rec: (u32, &str) = rec_server_master
            .recv()
            .expect("didn't get a message or something");
        let id: u32 = rec.0;
        let msg: &str = rec.1;
        let res_comm: &Comms = client_comms.get(&id).expect("no id match");
        let res_comm_channel = &res_comm.srv_send;
        if msg == "kill_me" {
            res_comm_channel
                .send((id, "kill"))
                .expect("something wrong");
            client_comms.remove(&id);
        } else if msg == "give" {
            res_comm_channel
                .send((id, user_list[(id % 4) as usize]))
                .expect("seomthing wrong");
        }
    }
}
