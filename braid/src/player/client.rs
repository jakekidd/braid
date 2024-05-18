use std::net::TcpStream;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use sha2::{Sha256, Digest}; // For cryptographic commitments.

// Structure to hold player data and their exploration mask for network communication.
#[derive(Serialize, Deserialize, Clone)]
struct PlayerData {
    id: usize,
    exploration_mask: Vec<Vec<bool>>,
    commitment: Vec<u8>, // Commitment of the current position.
}

// Structure to represent the player within the client application.
pub struct Player {
    id: usize,
    exploration_mask: Vec<Vec<bool>>,
    commitment: Vec<u8>, // Commitment of the current position.
}

impl Player {
    // Create a new player with a given ID and maze dimensions.
    pub fn new(id: usize, maze_width: usize, maze_height: usize) -> Self {
        let exploration_mask = vec![vec![false; maze_height]; maze_width];
        let commitment = vec![0; 32]; // Placeholder for the initial commitment.
        Player {
            id,
            exploration_mask,
            commitment,
        }
    }

    // Connect to the server.
    pub fn connect(&self, address: &str) -> TcpStream {
        TcpStream::connect(address).expect("Could not connect to the server")
    }

    // Send exploration data to the server and receive the current view of the maze.
    pub fn explore(&mut self, stream: &mut TcpStream) {
        // Commit the player's current exploration state.
        self.commit_current_state();

        let request = PlayerData {
            id: self.id,
            exploration_mask: self.exploration_mask.clone(),
            commitment: self.commitment.clone(),
        };

        // Serialize the request to JSON.
        let request_json = serde_json::to_vec(&request).unwrap();
        stream.write_all(&request_json).unwrap();

        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).unwrap();
        let response: Maze = serde_json::from_slice(&buffer[..bytes_read]).unwrap();

        // Process the received maze (e.g., display it, update the player's exploration).
        self.display_maze(&response);
    }

    // Commit the player's current exploration state using SHA-256.
    fn commit_current_state(&mut self) {
        let mut hasher = Sha256::new();
        for (x, row) in self.exploration_mask.iter().enumerate() {
            for (y, &discovered) in row.iter().enumerate() {
                if discovered {
                    hasher.update(format!("{},{}", x, y).as_bytes());
                }
            }
        }
        self.commitment = hasher.finalize().to_vec();
    }

    // Display the maze in ASCII format.
    fn display_maze(&self, maze: &Maze) {
        for row in &maze.grid {
            // Top line of cells.
            for cell in row {
                if cell.walls[0] {
                    print!("+---");  // Print the north wall.
                } else {
                    print!("+   ");
                }
            }
            println!("+");

            // Middle line of cells.
            for cell in row {
                if cell.walls[3] {
                    print!("|   ");  // Print the west wall.
                } else {
                    print!("    ");
                }
            }
            println!("|");
        }

        // Bottom line of the maze.
        for _ in 0..maze.width {
            print!("+---");
        }
        println!("+");
    }

    // Simulate player movement and exploration (for demo purposes).
    pub fn simulate_exploration(&mut self) {
        let mut rng = rand::thread_rng();
        let mut frontier = VecDeque::new();

        // Start from a random cell.
        let start_x = rng.gen_range(0..self.exploration_mask.len());
        let start_y = rng.gen_range(0..self.exploration_mask[0].len());
        self.exploration_mask[start_x][start_y] = true;
        frontier.push_back((start_x, start_y));

        while let Some((cx, cy)) = frontier.pop_front() {
            let neighbors = self.get_unvisited_neighbors(cx, cy);
            if !neighbors.is_empty() {
                frontier.push_back((cx, cy));
                let &(nx, ny) = neighbors.choose(&mut rng).unwrap();
                self.exploration_mask[nx][ny] = true;
                frontier.push_back((nx, ny));
            }
        }
    }

    // Get the list of unvisited neighbors of a cell.
    fn get_unvisited_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors = vec![];
        if x > 0 && !self.exploration_mask[x - 1][y] {
            neighbors.push((x - 1, y));
        }
        if x < self.exploration_mask.len() - 1 && !self.exploration_mask[x + 1][y] {
            neighbors.push((x + 1, y));
        }
        if y > 0 && !self.exploration_mask[x][y - 1] {
            neighbors.push((x, y - 1));
        }
        if y < self.exploration_mask[0].len() - 1 && !self.exploration_mask[x][y + 1] {
            neighbors.push((x, y + 1));
        }
        neighbors
    }
}

// Structure representing the maze (must match the server-side definition).
#[derive(Serialize, Deserialize, Clone)]
pub struct Maze {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<Cell>>,
}

// Structure representing a single cell in the maze (must match the server-side definition).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
    pub visited: bool,
    pub walls: [bool; 4], // Walls of the cell in the order [north, east, south, west].
}

fn main() {
    let mut player = Player::new(1, 10, 10); // Create a new player with ID 1 and a 10x10 maze.
    let mut stream = player.connect("127.0.0.1:7878"); // Connect to the server.

    // Simulate exploration for the demo.
    player.simulate_exploration();
    player.explore(&mut stream); // Send exploration data and receive the current maze view.
}
