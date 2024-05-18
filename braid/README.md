# Braid

A dungeon crawler game using a P2P state channel network and ZK. Dungeon nodes supply Players with the maze, and Players race to navigate, discovering and recording new parts of the maze as they go. At the center of the maze is a treasure comprised of the Players' antes for the game.

### File Structure

```
braid_maze_game/
├── Cargo.toml
├── run_demo.sh
├── src/
│   ├── main.rs
│   ├── dungeon/
│   │   ├── mod.rs
│   │   ├── maze.rs
│   │   └── server.rs
│   ├── player/
│   │   ├── mod.rs
│   │   ├── interface.rs
│   │   └── client.rs
│   ├── comm/
│   │   ├── mod.rs
│   │   ├── state_channel.rs
│   │   └── socket.rs
│   ├── log/
│   │   ├── mod.rs
│   │   ├── dungeon_log.rs
│   │   └── player_log.rs
│   └── blockchain/
│       ├── mod.rs
│       └── stub.rs
└── README.md
```
