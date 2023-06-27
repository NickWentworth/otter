use crate::board::ZobristHash;
use std::collections::HashMap;

use super::Score;

pub struct TranspositionData {
    pub score: Score,
    pub depth: u8,
}

/// Stores the evaluation of different board states, greatly reducing the search tree size
pub struct TranspositionTable {
    // TODO - hash map doesn't seem to be the right option here, can't figure out how to not re-hash the zobrist hash
    table: HashMap<ZobristHash, TranspositionData>,
}

impl TranspositionTable {
    /// Generates an empty transposition table
    pub fn new() -> TranspositionTable {
        TranspositionTable {
            table: HashMap::new(),
        }
    }

    /// Inserts data into the transposition table
    pub fn insert(&mut self, hash: ZobristHash, data: TranspositionData) {
        self.table.insert(hash, data);
    }

    /// Tries to fetch from the transposition table, given a current searching depth
    ///
    /// The depth is needed to prevent cases where a shallow evaluation is used instead of a deeper and more accurate evaluation
    pub fn get(&self, hash: ZobristHash, current_depth: u8) -> Option<&TranspositionData> {
        let data = self.table.get(&hash);

        // only return data if it is as deep (or deeper) of an evaluation that is being asked
        if data?.depth >= current_depth {
            data
        } else {
            None
        }
    }
}
