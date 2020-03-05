# Rusty Mancala
![rusty mancala](./img/rust-mancala.jpg "")

### Overview

Rusty Mancala is a basic implementation of the standard rules of [Mancala](https://en.wikipedia.org/wiki/Mancala).  It 
is mostly an exercise to learn the Rust programming language and TCP communication protocols.

### How to Use

1. Clone this repo
2. Run the binary in client mode
    - `$ cargo run c`
3. Enter host: `ec2-52-11-55-180.us-west-2.compute.amazonaws.com` 
4. Enter port: `4567`
5. Look for a game, change your nickname, or start your own game!

##### Example usage:
```shell script
$ cargo run c
    Finished dev [unoptimized + debuginfo] target(s) in 0.48s
     Running `target\debug\rusty-mancala.exe c`
run client
Enter a host: ec2-52-11-55-180.us-west-2.compute.amazonaws.com
Enter a port: 4567

    Now connected to: ec2-52-11-55-180.us-west-2.compute.amazonaws.com:4567.
    Your current nickname is: new_nick.
    Welcome to Mancala.  Please select one of the following options:
        (1) Change Nickname
        (2) List Available Games
        (3) List Active Users
        (4) Start New Game
        (5) Join Game
        (6) Disconnect

    Enter your choice: 3

server response: Active Users:
user_9
user_10
user_2
user_1
user_8
user_3
user_4
new_nick
user_5
```

##### Rules of Mancala
If you are unfamiliar with the rules of Mancala, please see 
[this lovely instructables article](https://www.instructables.com/id/How-to-play-MANCALA/).

### Playing Rusty Mancala

After creation your own game, you'll be put into a holding pattern until another user joins your game:

```shell script
    Waiting for another player...
```

When you get another player, player 1 will be presented with a screen showing their options:

```shell script
Current game state:
           #13: 4 | #12: 4 | #11: 4 | #10: 4 |  #9: 4 | #8: 4
        0 --------+--------+--------+--------+--------+-------- 0
            #1: 4 |  #2: 4 |  #3: 4 |  #4: 4 |  #5: 4 | #6: 4
Player 1, enter your move (1 - 6)
```

Player 1 then selects a slot to move their stones around the board from.  Meanwhile, player two is awaiting their turn 
patiently:

```shell script
Current game state:
            #6: 4 |  #5: 4 |  #4: 4 |  #3: 4 |  #2: 4 | #1: 4
        0 --------+--------+--------+--------+--------+-------- 0
            #8: 4 |  #9: 4 | #10: 4 | #11: 4 | #12: 4 | #13: 4


        Waiting for my turn...
```

Player 1 can make a move, and if it results in the turns changing, then it will be player 2's turn.  Let's say player 1 
moves slot 5, which does not result in a turn change.

Now player 1 sees:

```shell script
Current game state:
           #13: 4 | #12: 4 | #11: 4 | #10: 4 |  #9: 5 | #8: 5
        0 --------+--------+--------+--------+--------+-------- 1
            #1: 4 |  #2: 4 |  #3: 4 |  #4: 4 |  #5: 0 | #6: 5


        Waiting for my turn...
```

The board's changes are reflected for player 2 and it is now their turn!

```shell script
Current game state:
            #6: 5 |  #5: 0 |  #4: 4 |  #3: 4 |  #2: 4 | #1: 4
        1 --------+--------+--------+--------+--------+-------- 0
            #8: 5 |  #9: 5 | #10: 4 | #11: 4 | #12: 4 | #13: 4
Player 2, enter your move (8 - 13)

```

This proceeds until the game is finished, at which point both players are returned to the lobby.

## Details

This project is largely implemented via its [TcpStream](https://doc.rust-lang.org/std/net/struct.TcpStream.html) Struct 
in the standard library.

### Server

### Client

## Protocol

A large part of the communication between the client and the server is over a serialized messaging protocol.  The 
protocol is loosely defined below.  See the [Protocol](./protocol.txt) documentation for more details.

#### Message Object

```
{
    status:         Status,
    headers:        Headers,
    command:        Commands,
    game_status:    GameStatus,
    data:           String,
    game_state:     GameState,
}
```

The message object is the serialized struct that is sent in between the client and the server and drives all action. It
is used both in the game and outside of the game.  The general workflow is :
    
    - Client collects input from the user
    - User input is translated into a `Message` struct 
    - Client serializes message and sends to the server
    - Server ingests message, deserializes into a `Message` struct
    - Server takes appropriate action based on message
    - Server generates message to send to client, serializes it
    - Server sends message to client
    - Client ingests server response and prompts user for input
    - Repeat
    
The fields of the message are described below.

##### Status

```
{
    Ok,
    NotOk,
}
```

Status is an `enum` containing two values: `Ok` and `NotOk`.  These are used to communicate errors or successful 
actions for server or client.

##### Headers

```
{
    Read,
    Write,
    Response,
}
```

##### Commands

```
{
    InitSetup,
    SetNick,
    ListGames,
    ListUsers,
    MakeNewGame,
    JoinGame,
    LeaveGame,
    GetCurrentGamestate,
    MakeMove,
    GameIsOver,
    KillMe,
    KillClient,
    Reply,
}
```

##### Game Status

```
{
    InGame,
    NotInGame
}
```

##### Data

Data is a text field that contains metadata about the message sent.  It can be things like the nickname that the user has
chosen, or an error message that is returned to the client when the server chokes on something.

Examples:

```

```

##### Game State

```
{
    player_one:             int,
    player_two:             int,
    game_name:              String,
    game_id:                int,
    game_board:             [],
    player_one_goal_slot:   int,
    player_two_goal_slot:   int,
    player_one_turn:        bool,
    active:                 bool,
}
```

### Deployment

Server is deployed on AWS.  Connect by building the client, and connecting to host 
`ec2-52-11-55-180.us-west-2.compute.amazonaws.com` on port `4567`.

## Authors

* **Belén Bustamante** - [rooneyshuman](https://github.com/rooneyshuman)
* **Logan Ballard** - [loganballard](https://github.com/loganballard)

## License

Distributed under the MIT License. See [LICENSE](/LICENSE) for more information.
