#!/bin/bash

# Start the Dungeon node
cargo run --bin dungeon &

# Start the Player node
cargo run --bin player &

# Wait for all background processes to finish
wait
