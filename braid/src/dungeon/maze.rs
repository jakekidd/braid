use rand::Rng;
use std::collections::VecDeque;
use sha2::{Sha256, Digest}; // For cryptographic commitments.

// Representation of a single cell in the maze.
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub x: usize,               // X coordinate of the cell.
    pub y: usize,               // Y coordinate of the cell.
    pub visited: bool,          // Whether the cell has been visited during maze generation.
    pub walls: [bool; 4],       // Walls of the cell in the order [north, east, south, west].
}

impl Cell {
    pub fn new(x: usize, y: usize) -> Self {
        Cell {
            x,
            y,
            visited: false,     // Used in the initial generation (see Maze.generate).
            walls: [true; 4],   // All walls are initially present.
        }
    }
}

// Representation of the maze.
pub struct Maze {
    pub width: usize,           // Width of the maze.
    pub height: usize,          // Height of the maze.
    pub grid: Vec<Vec<Cell>>,   // 2D grid of cells.
}

impl Maze {
    pub fn new(width: usize, height: usize) -> Self {
        let mut grid = vec![];
        for x in 0..width {
            let mut row = vec![];
            for y in 0..height {
                // Create cells, which will have all walls intact.
                row.push(Cell::new(x, y));
            }
            grid.push(row);
        }
        Maze { width, height, grid }
    }

    // Generate the maze using Prim's Algorithm.
    pub fn generate(&mut self) {
        let mut rng = rand::thread_rng();
        let mut frontier = VecDeque::new();

        // Start from a random cell.
        let start_x = rng.gen_range(0..self.width);
        let start_y = rng.gen_range(0..self.height);
        self.grid[start_x][start_y].visited = true;
        frontier.push_back((start_x, start_y));

        while let Some((cx, cy)) = frontier.pop_front() {
            let neighbors = self.get_unvisited_neighbors(cx, cy);
            if !neighbors.is_empty() {
                // Re-add the current cell if it has unvisited neighbors.
                frontier.push_back((cx, cy));
                // Choose a random unvisited neighbor.
                let &(nx, ny) = neighbors.choose(&mut rng).unwrap();
                // Remove the wall between the current cell and the neighbor.
                self.remove_wall(cx, cy, nx, ny);
                // Mark the neighbor as visited.
                self.grid[nx][ny].visited = true;
                // Add the neighbor to the frontier.
                frontier.push_back((nx, ny));
            }
        }
    }

    // Get the list of unvisited neighbors of a cell.
    fn get_unvisited_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors = vec![];
        if x > 0 && !self.grid[x - 1][y].visited {
            neighbors.push((x - 1, y));  // Add the west neighbor.
        }
        if x < self.width - 1 && !self.grid[x + 1][y].visited {
            neighbors.push((x + 1, y));  // Add the east neighbor.
        }
        if y > 0 && !self.grid[x][y - 1].visited {
            neighbors.push((x, y - 1));  // Add the north neighbor.
        }
        if y < self.height - 1 && !self.grid[x][y + 1].visited {
            neighbors.push((x, y + 1));  // Add the south neighbor.
        }
        neighbors
    }

    // Remove the wall between two adjacent cells.
    fn remove_wall(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        if x1 == x2 {
            if y1 > y2 {
                self.grid[x1][y1].walls[0] = false;  // Remove north wall of (x1, y1).
                self.grid[x2][y2].walls[2] = false;  // Remove south wall of (x2, y2).
            } else {
                self.grid[x1][y1].walls[2] = false;  // Remove south wall of (x1, y1).
                self.grid[x2][y2].walls[0] = false;  // Remove north wall of (x2, y2).
            }
        } else if y1 == y2 {
            if x1 > x2 {
                self.grid[x1][y1].walls[3] = false;  // Remove west wall of (x1, y1).
                self.grid[x2][y2].walls[1] = false;  // Remove east wall of (x2, y2).
            } else {
                self.grid[x1][y1].walls[1] = false;  // Remove east wall of (x1, y1).
                self.grid[x2][y2].walls[3] = false;  // Remove west wall of (x2, y2).
            }
        }
    }

    // Generate a masked subset of the maze based on the player's exploration.
    pub fn get_masked_maze(&self, mask: &Vec<Vec<bool>>) -> Maze {
        let mut masked_grid = vec![];
        for x in 0..self.width {
            let mut row = vec![];
            for y in 0..self.height {
                if mask[x][y] {
                    // Add the cell if it's visible in the mask.
                    row.push(self.grid[x][y]);
                } else {
                    // Add a new cell with all walls intact if it's not visible.
                    row.push(Cell::new(x, y));
                }
            }
            masked_grid.push(row);
        }
        Maze {
            width: self.width,
            height: self.height,
            grid: masked_grid,
        }
    }

    // Commit the maze generation solution path.
    pub fn commit_solution_path(&self, solution_path: &[(usize, usize)]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        for &(x, y) in solution_path {
            hasher.update(format!("{},{}", x, y).as_bytes());
        }
        hasher.finalize().to_vec()
    }

    // Display the maze in ASCII format.
    pub fn display(&self) {
        for row in &self.grid {
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
        for _ in 0..self.width {
            print!("+---");
        }
        println!("+");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_generation() {
        let mut maze = Maze::new(10, 10);
        maze.generate();
        // Check that all cells are visited.
        assert!(maze.grid.iter().all(|row| row.iter().all(|cell| cell.visited)));
    }

    #[test]
    fn test_masked_maze() {
        let mut maze = Maze::new(5, 5);
        maze.generate();
        let mask = vec![
            vec![true, false, false, false, false],
            vec![true, true, false, false, false],
            vec![false, true, false, false, false],
            vec![false, true, true, false, false],
            vec![false, false, true, true, false],
        ];
        let masked_maze = maze.get_masked_maze(&mask);
        assert!(masked_maze.grid[0][0].visited); // Check that visible cells are visited.
        assert!(!masked_maze.grid[0][1].visited); // Check that masked cells are not visited.
    }

    #[test]
    fn test_commit_solution_path() {
        let maze = Maze::new(5, 5);
        let solution_path = vec![(0, 0), (0, 1), (1, 1), (2, 1), (2, 2)];
        let commitment = maze.commit_solution_path(&solution_path);
        assert_eq!(commitment.len(), 32); // Check the length of the SHA-256 hash.
    }
}
