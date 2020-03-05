# Rusty Mancala
![rusty mancala](./img/rust-mancala.jpg "")

### Overview

Rusty Mancala is a basic implementation of the standard rules of [Mancala](https://en.wikipedia.org/wiki/Mancala).  It is mostly an exercise to learn the Rust programming language and TCP communication protocols.

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
If you are unfamiliar with the rules of Mancala, please see [this lovely instructables article](https://www.instructables.com/id/How-to-play-MANCALA/).

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

Player 1 then selects a slot to move their stones around the board from.  Meanwhile, player two is awaiting their turn patiently:

```shell script
Current game state:
            #6: 4 |  #5: 4 |  #4: 4 |  #3: 4 |  #2: 4 | #1: 4
        0 --------+--------+--------+--------+--------+-------- 0
            #8: 4 |  #9: 4 | #10: 4 | #11: 4 | #12: 4 | #13: 4


        Waiting for my turn...
```

Player 1 can make a move, and if it results in the turns changing, then it will be player 2's turn.  Let's say player 1 moves slot 5, which does not result in a turn change.

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


## Technical Details



##### Deployment

Server is deployed on AWS.  Connect by building the client, and connecting to host `ec2-52-11-55-180.us-west-2.compute.amazonaws.com` on port `4567`.

## Authors

* **Belén Bustamante** - [rooneyshuman](https://github.com/rooneyshuman)
* **Logan Ballard** - [loganballard](https://github.com/loganballard)

## License

Distributed under the MIT License. See [LICENSE](/LICENSE) for more information.
