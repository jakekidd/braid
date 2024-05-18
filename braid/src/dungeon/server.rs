use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use serde::{Serialize, Deserialize};
use crate::dungeon::maze::{Maze, Cell};
use crate::blockchain::state_channel::{StateChannel, State};
use secp256k1::{Secp256k1, SecretKey, Signature, PublicKey};

/**
 * - Server Structure: Represents the server state with a shared maze and player data.
 * - New Server: Initializes the server with a generated maze.
 * - Add Player: Adds a new player to the server, initializing their exploration mask.
 * - Handle Client: Manages incoming player connections and processes their requests.
 * - Update Player Exploration: Updates the player's exploration mask.
 * - Get Player View: Returns the current view of the maze for the player based on their exploration mask.
 * - Update Treasure: Updates the treasure amount based on the current turn.
 * - Start Server: Listens for incoming connections and handles them in separate threads.
 * - Main Function: Initializes and starts the server.
 */

// Structure to hold player data and their exploration mask.
#[derive(Serialize, Deserialize, Clone)]
struct PlayerData {
    id: usize,
    exploration_mask: Vec<Vec<bool>>,
    commitment: Vec<u8>, // Commitment of the current position.
}

// Structure to represent the server state.
struct Server {
    maze: Arc<Mutex<Maze>>, // Shared maze between threads.
    players: Arc<Mutex<Vec<PlayerData>>>, // Shared player data between threads.
    state_channels: Arc<Mutex<HashMap<usize, StateChannel>>>, // State channels for each player.
    max_turns: usize, // Maximum number of turns allowed.
    current_turn: usize, // Current turn number.
    initial_treasure: f64, // Initial treasure amount.
    treasure: f64, // Current treasure amount.
}

impl Server {
    // Create a new server with a generated maze.
    fn new(maze_width: usize, maze_height: usize, max_turns: usize, initial_treasure: f64) -> Self {
        let mut maze = Maze::new(maze_width, maze_height);
        maze.generate();
        Server {
            maze: Arc::new(Mutex::new(maze)),
            players: Arc::new(Mutex::new(Vec::new())),
            state_channels: Arc::new(Mutex::new(HashMap::new())),
            max_turns,
            current_turn: 0,
            initial_treasure,
            treasure: initial_treasure,
        }
    }

    // Add a new player to the server.
    fn add_player(&self, player_id: usize, player_address: &str, server_address: &str) {
        let maze = self.maze.lock().unwrap();
        let exploration_mask = vec![vec![false; maze.height]; maze.width];
        let player_data = PlayerData {
            id: player_id,
            exploration_mask,
            commitment: vec![],
        };
        self.players.lock().unwrap().push(player_data);
        self.state_channels.lock().unwrap().insert(player_id, StateChannel::new(player_address, server_address));
    }

    // Handle incoming player connections.
    fn handle_client(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        while let Ok(bytes_read) = stream.read(&mut buffer) {
            if bytes_read == 0 {
                return; // Connection was closed.
            }
            let request: PlayerData = serde_json::from_slice(&buffer[..bytes_read]).unwrap();
            self.update_player_exploration(&request);
            let response = self.get_player_view(&request);
            let response_json = serde_json::to_vec(&response).unwrap();
            stream.write_all(&response_json).unwrap();

            self.current_turn += 1;
            self.update_treasure();
            if self.current_turn >= self.max_turns {
                println!("Max turns reached. Game over.");
                break;
            }
        }
    }

    // Update the player's exploration mask based on their request.
    fn update_player_exploration(&self, player_data: &PlayerData) {
        let mut players = self.players.lock().unwrap();
        if let Some(player) = players.iter_mut().find(|p| p.id == player_data.id) {
            player.exploration_mask = player_data.exploration_mask.clone();
            player.commitment = player_data.commitment.clone();
        }
    }

    // Get the current view of the maze for the player based on their exploration mask.
    fn get_player_view(&self, player_data: &PlayerData) -> Maze {
        let maze = self.maze.lock().unwrap();
        maze.get_masked_maze(&player_data.exploration_mask)
    }

    // Update the treasure amount based on the current turn.
    fn update_treasure(&mut self) {
        let fee_increase_start = self.max_turns / 2;
        if self.current_turn > fee_increase_start {
            let additional_fee = (self.current_turn - fee_increase_start) as f64 * 0.1;
            self.treasure = self.initial_treasure - additional_fee;
        }
    }

    // Start the server and listen for incoming connections.
    fn start(&self, address: &str) {
        let listener = TcpListener::bind(address).unwrap();
        println!("Server listening on {}", address);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let server = self.clone();
                    thread::spawn(move || {
                        server.handle_client(stream);
                    });
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    }
}

// Implement Clone for Server to allow cloning within threads.
impl Clone for Server {
    fn clone(&self) -> Self {
        Server {
            maze: Arc::clone(&self.maze),
            players: Arc::clone(&self.players),
            state_channels: Arc::clone(&self.state_channels),
            max_turns: self.max_turns,
            current_turn: self.current_turn,
            initial_treasure: self.initial_treasure,
            treasure: self.treasure,
        }
    }
}

fn main() {
    let server = Server::new(10, 10, 100, 1000.0); // Initialize the server with a 10x10 maze, 100 max turns, and initial treasure of 1000.
    server.start("127.0.0.1:7878"); // Start the server on localhost port 7878.
}
