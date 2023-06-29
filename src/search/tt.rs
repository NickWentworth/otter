use crate::board::ZobristHash;
use std::mem::size_of;

// TODO - add buckets to allow multiple entries stored at a single index

const MB_SIZE: usize = 1024 * 1024;

/// Describes an entry in the transposition table, contains a hash for verification and some data along with it
#[derive(Clone, Copy, Default)]
pub struct Entry<D> {
    hash: ZobristHash,
    data: D,
}

/// Stores the evaluation of different board states, greatly reducing the search tree size
pub struct TranspositionTable<D> {
    table: Vec<Entry<D>>, // uses zobrist hashes to store scores
    capacity: usize,      // amount of scores to be stored in the table
    used: usize,          // amount of scores currently stored in the table

    // statistics
    total: usize,      // total access attempts
    hits: usize,       // total hits from accesses
    collisions: usize, // collisions on insert
}

/// Data type must be have a default value and be copy-able for pre-allocation and accessing later on
impl<D: Copy + Default> TranspositionTable<D> {
    /// Generates an empty transposition table with alloted size in MB
    pub fn new(mb: usize) -> TranspositionTable<D> {
        // calculate how many entries can be stored in the table
        let capacity = (mb * MB_SIZE) / size_of::<Entry<D>>();

        TranspositionTable {
            table: vec![
                Entry {
                    hash: 0,
                    data: D::default()
                };
                capacity
            ],
            capacity,
            used: 0,
            total: 0,
            hits: 0,
            collisions: 0,
        }
    }

    /// Inserts data into the transposition table
    pub fn insert(&mut self, hash: ZobristHash, data: D) {
        let index = self.hash_index(hash);

        let residing_hash = self.table[index].hash;

        if residing_hash == 0 {
            self.used += 1;
        } else if residing_hash != hash {
            self.collisions += 1;
        }

        self.table[index] = Entry { hash, data };
    }

    /// Tries to fetch from the transposition table, given a current searching depth
    ///
    /// The depth is needed to prevent cases where a shallow evaluation is used instead of a deeper and more accurate evaluation
    pub fn get(&mut self, hash: ZobristHash) -> Option<D> {
        let entry = self.table[self.hash_index(hash)];

        self.total += 1;

        if entry.hash == hash {
            self.hits += 1;
            Some(entry.data)
        } else {
            None
        }
    }

    /// Prints debug statistics for the table
    pub fn print_stats(&self) {
        println!("capacity: {}", self.capacity);
        println!(
            "entries (used %): {} ({:.2}%)",
            self.used,
            self.used as f32 / self.capacity as f32 * 100f32
        );

        println!("total accesses: {}", self.total);
        println!(
            "hits (rate %): {} ({:.2}%)",
            self.hits,
            self.hits as f32 / self.total as f32 * 100f32
        );

        println!("collisions: {}", self.collisions);
        println!();
    }

    /// Returns the index in the table of the given hash
    fn hash_index(&self, hash: ZobristHash) -> usize {
        (hash as usize) % self.capacity
    }
}
