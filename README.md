# Otter

Otter is a UCI chess engine built with Rust. It does not rely upon external chess-related crates, though many commonly used chess programming concepts are utilized within the engine.

## Usage

Otter implements the essentials of the UCI protocol, allowing it to be used with external GUIs. Currently, it has been tested and is functional with the [Lucas Chess](https://lucaschess.pythonanywhere.com/downloads) GUI. To run and play versus the engine:

1. Install [Rust](https://www.rust-lang.org/tools/install) and clone this repository
2. Build the project by running `cargo build --release` from the command line at the root of the project
3. Install Lucas Chess (or any other GUI supporting UCI that you prefer)
4. Within the chess GUI, navigate to the engine configuration settings and select the built otter executable, located at `/target/release/otter.exe`

Otter can also be run without a GUI to evaluate positions or search for best moves. To run otter through the command line:

1. Install [Rust](https://www.rust-lang.org/tools/install) and clone this repository
2. Run the project by running `cargo run --release` from the command line at the root of the project
3. Useful available commands include:
    - `position fen [FEN]` to load a position
    - `go` to search for the best move at the position
    - `display` to show the current board state

## Features

#### Currently Implemented
- Board & Move Generation
    - Bitboard board representation
    - Pre-computed move lookup tables for non-sliding pieces
    - Magic bitboard move lookup tables for sliding pieces
- Search
    - Iterative deepening approach using the alpha-beta searching algorithm
    - Quiescence search
    - Transposition table that uses Zobrist hashing
    - Move ordering using MVV-LVA (most valuable victim - least valuable attacker) and a basic principal variation implementation
- Tests
    - Move generation is tested using a perft function on various complicated positions

#### Todo's
- Openings database to allow more variation in the early-game
- Stronger evaluation that considers ideas such as pawn structure and king safety
- Pondering to search best responses while opponent is making moves
- Implement more UCI commands, including `go infinite` and `stop`
