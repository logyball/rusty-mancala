use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Headers {
    Read,
    Write,
    Response,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Status {
    Ok,
    NotOk,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Commands {
    SetNick,
    ListGames,
    ListUsers,
    MakeNewGame,
    Reply
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Msg {
    pub status: Status,
    pub headers: Headers,
    pub command: Commands,
    pub data: String
}

impl Msg {
    pub fn serialize(&self, buf: &mut [u8; 512]) {
        let encoded: Vec<u8> = bincode::serialize(&self).unwrap();
        for i in 0..encoded.len() {
            buf[i] = encoded[i];
        }
    }
}

#[test]
fn test_serialize_msg() {
    let msg1: Msg = Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::SetNick,
        data: "data".to_string()
    };
    let mut buf: [u8; 512] = [0; 512];
    msg1.serialize(&mut buf);
    let msg2: Msg = bincode::deserialize(&buf[..]).unwrap();
    assert_eq!(msg1, msg2);
}
