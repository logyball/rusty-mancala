# Mancala Protocol

##### Logan Ballard | Internetworking Protocols | 3/6/2020

### Overview

The Manacala protocol is a multi-purpose in nature.  The primary goal of the protocol is to support a fully-featured
client/server Mancala implementation that can be developed in any programming language.  It allows users to establish 
a communication framework for find creating games, finding other players, customizing their experience, and playing
Mancala against another player.  This protocol is designed to be implemented over TCP, but could theoretically be used
over any communication medium capable of transmitting arrays of bytes with a known length.

- [Outline](#outline)
- [Message](#message)
  * [Status](#status)
  * [Headers](#headers)
  * [Commands](#commands)
    + [SetNick](#setnick)
    + [ListGames](#listgames)
    + [ListUsers](#listusers)
    + [MakeNewGame](#makenewgame)
    + [JoinGame](#joingame)
    + [LeaveGame](#leavegame)
    + [GetCurrentGamestate](#getcurrentgamestate)
    + [MakeMove](#makemove)
    + [GameIsOver](#gameisover)
    + [KillMe](#killme)
    + [KillClient](#killclient)
    + [Reply](#reply)
  * [GameStatus](#gamestatus)
  * [Data](#data)
    + [Examples](#examples)
- [GameState](#gamestate)

### Outline

The protocol is implemented via several enumerated values, an arbitrary-length text field, and a representation of the 
game state.  Because Mancala is a relatively simple game, the entirety of the game can be modeled using primitive data
types that are easy to serialize and transport over established networking protocols. Although this project was originally
implemented in Rust, the mesage object can be implemented by any programming language that has the ability to serialize 
data into byte arrays using the format outlined by Rust's [bincode](https://github.com/servo/bincode) crate.

The following sections will outline the various fields and structures within the message that allow this functionality.

## Message

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

On a high level, the message object should be familiar to anyone who uses HTTP.  It containsthe basic building blocks for
a client-driven client/server application.  It contains information about the desired action or read the client wants to
perform, it is (almost) stateless on the client end, and it is asynchronous. The message object is the serialized struct 
that is used both in the game and outside of the game.  When not in a game, A client will communicate with these messages 
to ask the server is currently connected and which games are available.  A client can also request to change their nickname
as well as join games that are not full.  When the client is in a game, the messages are used to communicate game state as
well as moves the client would like to make.  The general workflow is:
    
    - Client collects input from the user
    - User input is translated into a `Message` struct 
    - Client serializes message and sends to the server as an array of bytes
    - Server ingests byte array, deserializes into a `Message` struct
    - Server takes appropriate action based on message
    - Server generates message to send to client, serializes it
    - Server sends serialized message as byte array to client
    - Client ingests server response, takes appropriate action, and prompts user for input
    - Repeat
    
The fields of the message are described below.

### Status

```
{
    Ok,
    NotOk,
}
```

Status is an `enum` containing two values: `Ok` and `NotOk`.  These are used to communicate errors or successful 
actions for server or client.  Both the client and server are expected to implement error handling for `NotOk` 
messages.

### Headers

```
{
    Read,
    Write,
    Response,
}
```

Various headers to indicate how a client or server should handle this message.  `Read` messages do not require any 
updates to data, so will be handled in a way that doesn't obtain locks or mess with writing.  Write data is handled
more carefully. Response data is information given from server to client.  These headers should be familiar to 
anyone used to HTTP protocol, as `Read` and `Write` loosely resemble `GET`, `POST`, wherease `Response` is analogous
to the HTTP response object type.

### Commands

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

The commands are used to drive action by the server or the client.  In general, the client will ask the server to do 
something, and the server will reply with a status that tells the client whether or not that action was successful.

C = Client
S = Server

| Command              | Direction | Description | payload |
|:----------------------:|:-----------:|:-----------|:---------|
| SetNick             | C -> S  | Client requests a change to their nickname | the new nickname |
| ListGames           | C -> S  | Request to return a list of open games | none |
| ListUsers           | C -> S  | Request to return a list of active users | none |
| MakeNewGame         | C -> S  | Create a new game and add me to it | text input game name |
| JoinGame            | C -> S  | Request to join an available game | the id of the game |
| LeaveGame           | C -> S  | Request to leave a game once joined | none |
| GetCurrentGamestate | C -> S  | Return the current board state | none |
| MakeMove            | C -> S  | While in game, move stones from a slot | The slot to move stones from |
| GameIsOver          | S -> C  | Response when game state has reached "finished" | none |
| KillMe              | C -> S  | Request to remove client from active lists | none |
| KillClient          | S -> C  | Response to end client TCP connection | none |
| Reply               | S -> C  | Generic reply format | varies |

#### SetNick

This command is passed from client to server to set the client's nickname.  The server is expected to check against
a database of known client's to make sure the intended nickname is globally unique.  If it is, the client should
be informed of their successful name change.  The server should track a mapping between connected clients and their
nicknames.  

The nickname should be passed as a string in the `data` field.

```
{
    status:         Status::Ok,
    headers:        Headers::Write,
    command:        Commands::SetNick,
    game_status:    GameStatus::NotInGame,
    data:           "nickname",
    game_state:     empty,
}
```

#### ListGames

This command is passed from client to server to retrieve a list of available games.  A game is considered available 
if it has less than 2 players in it, and it has not finished.  In responding to this command, a server is expected
to be able to find a list of games that fit that criteria and return them to the client.  If no such games are 
available, an appropriate error should be returned.  Consider using the `Status` field.

#### ListUsers

This command is passed from client to server to retrieve a list of active users by their nickname.  A user is 
considered active if they are currently connected to the server.  They can be in game or out of game.  This list
is useful for clients who would like to change their nicknames.  

#### MakeNewGame

This command is passed from the client to the server to indicate to the server that the client wants to create a 
new open game.  The client can optionally pass a name for the new game.  The server is expected to greate a new
`GameState` object and add the player who sent the message to it.

Example:

```
{
    status:         Status::Ok,
    headers:        Headers::Write,
    command:        Commands::MakeNewGame,
    game_status:    GameStatus::NotInGame,
    data:           "NewGame",
    game_state:     empty
}
```

Example Response:

```
{
    status:         Status::Ok,
    headers:        Headers::Response,
    command:        Commands::MakeNewGame,
    game_status:    GameStatus::InGame,
    data:           "NewGame",
    game_state:     {
                        player_one: 1,
                        player_two: 0,
                        game_name: "NewGame",
                        game_id: 1,
                        game_board: [0,4,4,4,4,4,4,0,4,4,4,4,4,4],
                        player_one_goal_slot: 7,
                        player_two_goal_slot: 0,
                        player_one_turn: true,
                        active: false,
                        game_over: false,
                    }
}
```

#### JoinGame

This command is sent from the client to the server when a client would like to join an open game.  The client must
pass an identifier to the game that allows the server to determine which game to add the client to.  This identifier
should be passed through the `data` field.

Example:

```
{
    status:         Status::Ok,
    headers:        Headers::Write,
    command:        Commands::JoinGame,
    game_status:    GameStatus::NotInGame,
    data:           "0",
    game_state:     empty
}
```

Example Successful Response:

```
{
    status:         Status::Ok,
    headers:        Headers::Response,
    command:        Commands::JoinGame,
    game_status:    GameStatus::InGame,
    data:           "Joined Game NewGame",
    game_state:     {
                        player_one: 1,
                        player_two: 2,
                        game_name: "NewGame",
                        game_id: 0,
                        game_board: [0,4,4,4,4,4,4,0,4,4,4,4,4,4],
                        player_one_goal_slot: 7,
                        player_two_goal_slot: 0,
                        player_one_turn: true,
                        active: true,
                        game_over: false,
                    }
}
```

Example Unsccessful Response:

```
{
    status:         Status::Ok,
    headers:        Headers::Response,
    command:        Commands::JoinGame,
    game_status:    GameStatus::NotInGame,
    data:           "Game is unavailable",
    game_state:     empty
}
```

#### LeaveGame

This command is issued from the client to the server from an in-game state to release the client from the game.  The
server is expected to update the game appropriately and notify the client requesting to leave as well as the client
that sharing the game (if there is another client in the game).

Example Response:
```
{
    status:         Status::Ok,
    headers:        Headers::Write,
    command:        Commands::GameIsOver,
    game_status:    GameStatus::NotInGame,
    data:           "Game Over - client id 1 left!",
    game_state:     empty
}
```

#### GetCurrentGamestate

This command is issued from the client to the server often.  Any time that the gamestate must be updated, which is 
many times (checking for the opponent making a move).  The client will often need to know when the game has changed
in order to determine whether the game has ended or whether the client's opponent has made a move.

#### MakeMove

This command is sent from the client to the server when a client is in a game.  While in a game, the client will 
need to make a move on their turn.  The move that will be made should be passed through the `data` field.  As the client
has access to the current game state at all times, the client should perform input validation before sending the
move to the server.

Example:

```
{
    status:         Status::Ok,
    headers:        Headers::Write,
    command:        Commands::MakeMove,
    game_status:    GameStatus::InGame,
    data:           "2",
    game_state:     {
                        player_one: 1,
                        player_two: 2,
                        game_name: "NewGame",
                        game_id: 0,
                        game_board: [0,4,4,4,4,4,4,0,4,4,4,4,4,4],
                        player_one_goal_slot: 7,
                        player_two_goal_slot: 0,
                        player_one_turn: true,
                        active: true,
                        game_over: false,
                    }
}
```

Example Response:

```
{
    status:         Status::Ok,
    headers:        Headers::Response,
    command:        Commands::MakeMove,
    game_status:    GameStatus::InGame,
    data:           "2",
    game_state:     {
                        player_one: 1,
                        player_two: 2,
                        game_name: "NewGame",
                        game_id: 0,
                        game_board: [0,4,0,5,5,5,5,0,4,4,4,4,4,4],
                        player_one_goal_slot: 7,
                        player_two_goal_slot: 0,
                        player_one_turn: false,
                        active: true,
                        game_over: false,
                    }
}
```

#### GameIsOver

This command is sent from the server to the client when the game is determined to be over.  According to the rules of 
Mancala, a game is over when either player can no longer make any legal moves (all of their slots are empty).  Because
the server is responsible for determining the state of the game, it should inform the client of the game end.  The
client should appropriately handle this by kicking the user back into the lobby.

Example Response:

```
{
    status:         Status::Ok,
    headers:        Headers::Response,
    command:        Commands::GameIsOver,
    game_status:    GameStatus::NotInGame,
    data:           "Game Over! Final Score:\n\tPlayer One: 24\n\tPlayer Two: 24\n",
    game_state:     {
                        player_one: 1,
                        player_two: 2,
                        game_name: "NewGame",
                        game_id: 0,
                        game_board: [24,0,0,0,0,0,0,24,0,0,0,0,0,0],
                        player_one_goal_slot: 7,
                        player_two_goal_slot: 0,
                        player_one_turn: false,
                        active: false,
                        game_over: true,
                    }
}
```

#### KillMe

This message is sent from the client to the server for the purposes of disconnection.  If a client disconnects via
the menu option, a message is sent explicitly, and the server is expected to respond.  If the client disconnects for 
other reasons (`ctrl+c`, act of God), this message should be generated internally for the server to appropriately
handle the client's unexpected absence (clean up nicknames/games the client belonged to).

When a server recieves a `KillMe` command, it should clean up all references to the client in shared data structures,
then respond to the client with a `KillClient` message, outlined below.

Example:

```
{
    status:         Status::Ok,
    headers:        Headers::Read,
    command:        Commands::KillMe,
    game_status:    GameStatus::NotInGame,
    data:           empty,
    game_state:     empty,
}
```

#### KillClient

This command is issued from the server to the client, indicating that the server has recieved the clients request
to be terminated, has appropriately cleaned up references to the client, and the client may now gracefully shut
down.

Example:

```
{
    status:         Status::Ok,
    headers:        Headers::Response,
    command:        Commands::KillClient,
    game_status:    GameStatus::NotInGame,
    data:           "Nick newnick successfully booted",
    game_state:     empty,
}
```

#### Reply

This a generic command issued by the server that the client can ignore.  It should be accompanied with data in
the `data` field that may be useful to the client, but in most circumstances simply reading the status header
should be enough for the client to understand what the context of the message is.  For example, when responding
to a `GetCurrentGameState` request, a server may reply with:

```
{
    status:         Status::Ok,
    headers:        Headers::Read,
    command:        Commands::Reply,
    game_status:    GameStatus::InGame,
    data:           "Current Game State",
    game_state:     {
                        player_one: 1,
                        player_two: 2,
                        game_name: "NewGame",
                        game_id: 0,
                        game_board: [24,0,0,0,0,0,0,24,0,0,0,0,0,0],
                        player_one_goal_slot: 7,
                        player_two_goal_slot: 0,
                        player_one_turn: false,
                        active: false,
                        game_over: true,
                    }
}
```

### GameStatus

```
{
    InGame,
    NotInGame
}
```

Game status drives certain code paths.  For the client, it determines which screens to render.  For the server, it 
determines how to route client messages.

### Data

Data is a text field that contains metadata about the message sent.  It can be things like the nickname that the user has
chosen, or an error message that is returned to the client when the server chokes on something.

#### Examples:

New Nickname "nick" Server Success Response:

```
{
    status:         Status::Ok,
    headers:        Headers::Response,
    command:        Commands::SetNick,
    game_status:    GameStatus::NotInGame,
    data:           "nick",
    game_state:     empty,
}
```

New Nickname Server Error Response:
```
{
    status:         NotOk,
    headers:        Response,
    command:        SetNick,
    game_status:    NotInGame,
    data:           "nickname already in use",
    game_state:     empty
}
```

## GameState

TODO 

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
    game_over:              bool,
}
```

