# Mancala Protocol

##### Logan Ballard | Internetworking Protocols | 3/6/2020

### Overview

The Manacala protocol is multi-purpose in nature.  The primary goal of the protocol is to support a fully-featured
client/server Mancala implementation that can be developed in any programming language.  It allows users to establish 
a communication framework for finding and creating games, finding other players, customizing their experience, and playing
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
  * [Fields](#fields)
    + [player_one](#player_one)
    + [player_two](#player_two)
    + [game_name](#game_name)
    + [game_id](#game_id)
    + [game_board](#game_board)
    + [player_one_goal_slot](#player_one_goal_slot)
    + [player_two_goal_slot](#player_two_goal_slot)
    + [player_one_turn](#player_one_turn)
    + [active](#active)
    + [game_over](#game_over)
  * [Functions](#functions)
    + [make_move](#make_move)
    + [capture](#capture)
    + [is_game_over](#is_game_over)
    + [collect_remaining](#collect_remaining)

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

On a high level, the message object should not be surprising to anyone who familiar with HTTP.  It contains the basic building blocks for
a client-driven client/server application.  It contains information about the desired action or read that the client wants to
perform, it is (almost) stateless on the client end, and it is asynchronous. The message object is the serialized struct 
that is used both in the game and outside of the game.  When not in a game, a client will communicate with these messages 
to ask the server what other users are currently active and which games are available.  A client can also request to change their nickname, to join games that are not full, or to create a new game.  When the client is in a game, the messages are used to communicate game state as
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

Various headers are implemented to indicate how a client or server should handle this message.  `Read` messages do not require any 
updates to data, so will be handled in a way that doesn't obtain locks or mess with writing.  Write data is handled
more carefully. Response data is information given from server to client.  These headers should be familiar to 
anyone used to HTTP protocol, as `Read` and `Write` loosely resemble `GET` and `POST`, wherease `Response` is analogous
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
something, and the server will reply with a status that tells the client whether or not that action was successful.  The server can additionally return data to the client.
Below is a summary of what each command does, followed by a detailed explaination.

C = Client
S = Server

| Command              | Direction | Description | Data |
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
a database of known client's names to make sure the intended nickname is globally unique.  If it is, the client should
be informed of their successful name change.  The server should track a mapping between connected clients and their
nicknames.  

The desired nickname should be passed as a string in the `data` field.

```
{
    status:         Ok,
    headers:        Write,
    command:        SetNick,
    game_status:    NotInGame,
    data:           "nickname",
    game_state:     empty,
}
```

#### ListGames

This command is passed from client to server to retrieve a list of available games.  A game is considered available 
if it has less than 2 players in it and is not finished.  In responding to this command, a server is expected
to be able to find a list of games that fit that criteria and return them to the client.  If no such games are 
available, an appropriate error should be returned.  Consider using the `Status` field.

#### ListUsers

This command is passed from client to server to retrieve a list of active users by their nickname.  A user is 
considered active if they are currently connected to the server.  They can be in game or out of game.  This list
is useful for clients who would like to change their nicknames and wish to avoid collisions.  

#### MakeNewGame

This command is passed from the client to the server to indicate to the server that the client wants to create a 
new open game.  The client can optionally pass a name for the new game.  The server is expected to greate a new
`GameState` object and add the player who sent the message to it.

Example:

```
{
    status:         Ok,
    headers:        Write,
    command:        MakeNewGame,
    game_status:    NotInGame,
    data:           "NewGame",
    game_state:     empty
}
```

Example Response:

```
{
    status:         Ok,
    headers:        Response,
    command:        MakeNewGame,
    game_status:    InGame,
    data:           "NewGame^0",
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
    status:         Ok,
    headers:        Write,
    command:        JoinGame,
    game_status:    NotInGame,
    data:           "0",
    game_state:     empty
}
```

Example Successful Response:

```
{
    status:         Ok,
    headers:        Response,
    command:        JoinGame,
    game_status:    InGame,
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

Example Unsuccessful Response:

```
{
    status:         Ok,
    headers:        Response,
    command:        JoinGame,
    game_status:    NotInGame,
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
    status:         Ok,
    headers:        Write,
    command:        GameIsOver,
    game_status:    NotInGame,
    data:           "Game Over - client id 1 left!",
    game_state:     empty
}
```

#### GetCurrentGamestate

This command is issued from the client to the server often.  Any time that the gamestate must be updated, which is 
many times (checking for the opponent making a move), this command should be used.  The client will often need to 
know when the game has changed in order to determine whether the game has ended or whether the client's opponent 
has made a move.

Example Request:

```
{
    status:         Ok,
    headers:        Read,
    command:        GetCurrentGamestate,
    game_status:    InGame,
    data:           empty,
    game_state:     empty,
}
```

Example Response:

```
{
    status:         Ok,
    headers:        Response,
    command:        Reply,
    game_status:    InGame,
    data:           "Current Game State",
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

#### MakeMove

This command is sent from the client to the server when a client is in a game.  While in a game, the client will 
need to make a move on their turn.  The move that will be made should be passed through the `data` field.  As the client
has access to the current game state at all times, the client should perform input validation before sending the
move to the server.

Example:

```
{
    status:         Ok,
    headers:        Write,
    command:        MakeMove,
    game_status:    InGame,
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
    status:         Ok,
    headers:        Response,
    command:        MakeMove,
    game_status:    InGame,
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
Mancala, a game is over when one player can no longer make any legal moves (all of their slots are empty).  Because
the server is responsible for determining the state of the game, it should inform the client of the game end.  The
client should appropriately handle this by kicking the user back into the lobby.

Example Response:

```
{
    status:         Ok,
    headers:        Response,
    command:        GameIsOver,
    game_status:    NotInGame,
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
    status:         Ok,
    headers:        Read,
    command:        KillMe,
    game_status:    NotInGame,
    data:           empty,
    game_state:     empty,
}
```

Example Response:

```
{
    status:         Ok,
    headers:        Response,
    command:        KillClient,
    game_status:    NotInGame,
    data:           "Nick newnick successfully booted",
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
    status:         Ok,
    headers:        Response,
    command:        KillClient,
    game_status:    NotInGame,
    data:           "Nick newnick successfully booted",
    game_state:     empty,
}
```

#### Reply

This a generic command issued by the server that the client can often ignore.  It should be accompanied with data in
the `data` field that may be useful to the client, but in most circumstances simply reading the status header
should be enough for the client to understand what the context of the message is.  For example, when responding
to a `GetCurrentGameState` request, a server may reply with:

```
{
    status:         Ok,
    headers:        Response,
    command:        Reply,
    game_status:    InGame,
    data:           "Current Game State",
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
    status:         Ok,
    headers:        Response,
    command:        SetNick,
    game_status:    NotInGame,
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

The gamestate object is passed from the server to the client in response to the client's requests to view the status of
the game, or in response to the client's actions.  The actual shared state of the game between the two players is managed
on the server and the clients are simply interacting with it through requests.  The fields of the game state and some
suggested function implmentations are outlined below.

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

### Fields

#### player_one

This field is an identifier for the first player in the game.  It is suggested to use a globally unique integer ID that
is assigned upon TCP connection.

#### player_two

This field is an identifier for the second player in the game.  It is suggested to use a globally unique integer ID that
is assigned upon TCP connection.

#### game_name

This field is a text string that is given to the server in the `MakeNewGame` command described above.

#### game_id

This field an integer ID automatically assigned by the server upon creation of a new game.  It is used to track which 
players are in which game and for requests from the client.

#### game_board

This field is an array that represents the the actual slots and "Mancala" (goal slot) of the mancala board.  Each index represents a 
slot on the board, and its values are the amount of stones in that particular slot.

#### player_one_goal_slot

This field is an integer that corresponds to the index of player one's "Mancala", or goal slot.  It also serves to track
their score.

#### player_two_goal_slot

This field is an integer that corresponds to the index of player two's "Mancala", or goal slot. It also serves to track
their score.

#### player_one_turn

This field is a boolean set to `true` if it is the first player's turn and `false` otherwise.

#### active

This field is a boolean set to `true` if two players are currently in the game and playing with one another.  Otherwise 
it is set to `false`.  It is useful for tracking available games that players can join.

#### game_over

This field is a boolean set to `false` until it is determined that at least one player has no more legal moves, at which
point it is set to `true`.  It is useful for determining the end of a game for both players simultaneously.

### Functions

These are several functions that should be implemented on the game state object to facilitate gameplay.  Although they are
not strictly necessary for the protocol, they will be useful for an actual implementation of a Mancala client/server 
application that uses this protocol.

#### make_move

This function takes a slot number and moves the stones appropriately across the board, taking care to add them to the
correct mancala if necessary, and skip the opponents mancala if necessary.

#### capture

The capture functionality can optionally be implemented.  See [this description](https://mancala.fandom.com/wiki/Capturing_(game_mechanism))
of capturing for more details.

#### is_game_over

This function should asses whether or not there is a legal move for the current player and end the game if there isn't.

#### collect_remaining

This function will be triggered after the game is over to "sweep" any remaining stones on either player's side into their
mancala and add them to their overall score.