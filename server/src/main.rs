extern crate websocket;
extern crate rand;

mod gamestate;

use std::thread;
use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;
use websocket::header::WebSocketProtocol;
use gamestate::*;
use rand::Rng;
use std::str;
use std::collections::HashSet;

// This is the main function for the GoMind Server. Here, we accept
// connections from new clients, and generally manage networking
// stuff. The logic of handling messages from the client is in
// handle_message, and playing the game is in make_move.
fn main() {
    // Print a little message to let the user know we've initialized.
    println!("Starting server on 192.168.0.2:2794");

    // Open a connection, binding to the outward facing network
    // interface.
    let server = match Server::bind("192.168.0.2:2794") {
        Ok(server) => server,
        Err(_) => {
            println!("Could not start server! Exiting.");
            return;
        },
    };
    // Accept each connection and have it spawn it's own thread.
    for connection in server {
        thread::spawn(move || {
            // Open up the request
            let request = match connection {
                Ok(connect) => match connect.read_request() {
                    Ok(req) => req,
                    Err(err) => {
                        println!("Couldn't read request: {}\nGiving up on this connection.", err);
                        return;
                    },
                },
                Err(err) => {
                    println!("Couldn't open connection: {}\nGiving up on this connection.", err);
                    return;
                }
            };
            
            let headers = request.headers.clone();
            match request.validate() {
                Ok(_) => (),
                Err(err) => {
                    println!("Couldn't validate request: {}\nGiving up on this connection.", err);
                    return;
                }
            }

            let mut response = request.accept();

            if let Some(&WebSocketProtocol(ref protocols)) = headers.get() {
                if protocols.contains(&("rust-websocket".to_string())) {
                    response.headers.set(WebSocketProtocol(vec!["rust-websocket".to_string()]));
                }
            }

            let mut client = match response.send() {
                Ok(client) => client,
                Err(err) => {
                    println!("Couldn't extract client: {}\nGiving up on this connection.", err);
                    return;
                }
            };
            let ip = match client.get_mut_sender().get_mut().peer_addr() {
                Ok(addr) => addr,
                Err(err) => {
                    println!("Couldn't extract client address: {}\nGiving up on this connection.", err);
                    return;
                }
            };

            // We've connected! Print a message indicating the ip and everything.
            println!("Connection from {}", ip);

            let mut state = GameState{board: [[TileState::Empty; BOARD_SIZE]; BOARD_SIZE]};

            // Split the connection into a sender and a receiver.
            let (mut sender, mut receiver) = client.split();

            // Handle each message as it comes.
            for message in receiver.incoming_messages() {
                let message: Message = match message {
                    Ok(msg) => msg,
                    Err(err) => {
                        println!("Couldn't open message: {}\nIgnoring this message.", err);
                        continue;
                    }
                };

                match message.opcode {
                    Type::Close => {
                        let message = Message::close();
                        match sender.send_message(&message) {
                            Ok(_) => (),
                            Err(_) => {
                                println!("Couldn't send close message to client. Ignoring.");       
                            }
                        }
                        println!("Client {} disconnected", ip);
                        return;
                    },
                    Type::Ping => {
                        let message = Message::pong(message.payload);
                        match sender.send_message(&message) {
                            Ok(_) => (),
                            Err(_) => {
                                println!("Couldn't respond to ping. Ignoring.");
                            }
                        }
                    },
                    Type::Text => {
                        let message_bytes: &[u8] = &message.payload.clone();
                        let message: String = str::from_utf8(message_bytes).expect("Ignoring message: couldn't parse bytes.");
                        println!("Received message:\n{}", message);
                        
                        let preresponse = process_move(message, &mut state);
                        sender.send_message(&prerepsonse).expect("Disconnecting client: couldn't send preresponse.");

                        let repsonse = make_move(&mut state);
                        sender.send_message(&response).expect("Disconnecting client: couldn't send response.");
                        
                        match response.opcode {
                            Type::Close => return,
                            Type::Text => println!("Responding:\n{}", str::from_utf8(&response.payload.clone()).unwrap()),
                            _ => (),
                        };
                    },
                    _ => sender.send_message(&message).unwrap(),

                }
            }
        });
    }
}

fn process_move<'a>(message: String, state: &mut GameState) -> Message<'a> {
    
}

fn parse_move<'a>(message: String) -> Move {
    let (move_type, pos) = message.split(":").collect::<Ve
    let move_coords = message.split(",").collect::<Vec<&str>>();
    let (x, y) = (move_coords[0].parse::<usize>()
                  .expect(format!("Ignoring move: couldn't parse x coordinate \"{}\".", move_coords[0])),
                  move_coords[1].parse::<usize>()
                  .expect(format!("Ignoring move: couldn't parse y coordinate \"{}\".", move_coords[1])));
    
}
// Handle a single client message. This does all the stuff essential
// to game logic and protocol, without being part of the decision making.
fn handle_message<'a>(message: String, state: &mut GameState) -> Message<'a> {
    // Parse the move we were sent
    let move_coords = message.split(",").collect::<Vec<&str>>();
    let x = match move_coords[0].parse::<usize>(){
        Ok(x) => x,
        Err(_) => {
            println!("Couldn't parse x coordinate: {}\nIgnoring message.", move_coords[0]);
            // This will send the old state right back to them.
            return Message::text(state.to_string());
        }
    };
    let y = match move_coords[1].parse::<usize>() {
        Ok(y) => y,
        Err(_) => {
            println!("Couldn't parse y coordinate: {}\nIgnoring message.", move_coords[1]);
            // This will send the old state right back to them.
            return Message::text(state.to_string());
        }
    };

    // Check that the move is legal.
    if x > BOARD_SIZE || y > BOARD_SIZE {
        println!("Illegal move: outside of board boundries. Move: ({},{})", x, y);
        return Message::close();
    }

    match state.board[x][y] {
        TileState::Empty => (),
        TileState::White => {
            println!("Illegal move: there's already a piece there.");
            return Message::close();
        },
        TileState::Black => {
            println!("Illegal move: there's already a piece there.");
            return Message::close();
        },
    }

    // Set the board state to reflect the move
    println!("Placing white piece at ({},{})", x, y);
    state.board[x][y] = TileState::White;

    handle_captures(state);

    if state.is_full() {
        println!("The board is full. Game is over.");
        return Message::close();
    }

    // Make a move in response
    let (newmove_x, newmove_y) = make_move(state);

    println!("Placing black piece at ({},{})", newmove_x, newmove_y);
    state.board[newmove_x][newmove_y] = TileState::Black;

    handle_captures(state);

    if state.is_full() {
        println!("The board is full. Game is over.");
        return Message::close();
    }

    // Return the new state of the board.
    return Message::text(state.to_string());
}

// Make a move. Right now, we just look for a random empty space and
// move there.
fn make_move(state: &GameState) -> (usize, usize) {
    let mut rng = rand::thread_rng();
    let move_x = rng.gen_range(0, BOARD_SIZE);
    let move_y = rng.gen_range(0, BOARD_SIZE);

    match state.board[move_x][move_y] {
        TileState::Empty => return (move_x, move_y),
        _ => return make_move(state),
    }
}

fn handle_captures(state: &mut GameState) {
    let liberties = get_liberties(state);
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            if liberties[x][y] == 0 {
                state.board[x][y] = TileState::Empty;
            }
        }
    }
}

fn get_liberties(state: &GameState) -> [[usize; BOARD_SIZE]; BOARD_SIZE] {
    println!("Assessing liberties.");
    let mut liberties: [[Option<usize>; BOARD_SIZE]; BOARD_SIZE] = [[None; BOARD_SIZE]; BOARD_SIZE];
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            match liberties[x][y] {
                Some(_) => (),
                None =>
                    match state.board[x][y] {
                        TileState::Empty => liberties[x][y] = Some(0),
                        _ => {
                            let cur_state = state.board[x][y];
                            println!("Exploring {} group starting at ({},{}).",
                                     if cur_state == TileState::White { "white" } else { "black" }, x, y);
                            let mut group_members = HashSet::new();
                            let mut group_liberties = HashSet::new();
                            let mut to_process = HashSet::new();

                            to_process.insert((x,y));

                            while !to_process.is_empty() {
                                let (nx, ny) = *to_process.iter().next().unwrap();
                                to_process.remove(&(nx, ny));
                                if group_members.contains(&(nx, ny)) { continue; }
                                println!("Processing tile ({},{}).", nx, ny);
                                group_members.insert((nx, ny));
                                for neighbor in
                                    vec![(nx as i32 - 1, ny as i32),
                                         (nx as i32 + 1, ny as i32),
                                         (nx as i32, ny as i32 - 1),
                                         (nx as i32, ny as i32 + 1)]
                                    .into_iter() {
                                        if neighbor.0 < BOARD_SIZE as i32 && neighbor.0 >= 0 &&
                                            neighbor.1 < BOARD_SIZE as i32 && neighbor.1 >= 0 {
                                                let tilestate = state.get_tile((neighbor.0 as usize, neighbor.1 as usize));
                                                if tilestate == cur_state {
                                                    to_process.insert((neighbor.0 as usize, neighbor.1 as usize));
                                                } else if tilestate == TileState::Empty {
                                                    group_liberties.insert(neighbor);
                                                    println!("Liberty found at ({},{}).", neighbor.0, neighbor.1);
                                                }
                                        }
                                }
                            }

                            for pos in group_members.into_iter() {
                                let (px, py) = pos;
                                liberties[px][py] = Some(group_liberties.len());
                            }
                        }
                    },
            }
        }
    }
    println!("");
    let mut lib_array: [[usize; BOARD_SIZE]; BOARD_SIZE] = [[0; BOARD_SIZE]; BOARD_SIZE];
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            match liberties[x][y] {
                Some(liberty_count) => lib_array[x][y] = liberty_count,
                None => (),
            }
        }
    }
    return lib_array;
}
