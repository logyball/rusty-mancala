use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn do_message_actions(snd_channel: mpsc::Sender<&str>, rec_channel: mpsc::Receiver<&str>) {
    thread::sleep(Duration::from_secs(1));
    loop {
        let mut rec: &str = rec_channel.recv().expect("didn't get a message or something");
        if rec == "kill" {
            return ()
        }
        println!("client recieved: {}", rec);
        snd_channel.send("g").unwrap();
    }
}

fn main () {
    let mut user_list: Vec<&str> = vec![];
    user_list.push("logan");
    user_list.push("belen");

    // let (send_server, rec_client) = mpsc::channel();
    let (send_client, rec_server) : (mpsc::Sender<&str>, mpsc::Receiver<&str>) = mpsc::channel();
    let (send_server, rec_client) : (mpsc::Sender<&str>, mpsc::Receiver<&str>) = mpsc::channel();

    let child = thread::spawn(move || {
        send_client.send("g").unwrap();
        do_message_actions(send_client, rec_client);
    });

    let mut cur_user = 0;
    loop {
        let mut msg = rec_server.recv().expect("something fd up");
        if msg == "g" {
            send_server.send(user_list[cur_user]).unwrap();
            cur_user += 1;
        }
        if cur_user >= user_list.len() {
            send_server.send("kill").unwrap();
            break;
        }
    }
    child.join();
    // instantiates something that is "shared"
    // spins off a process with channels
    // process asks server for shared data
        // print out shared data
        // send update message
    // server updates the shared data
}