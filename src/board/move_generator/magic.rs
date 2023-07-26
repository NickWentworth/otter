use crate::{
    board::move_generator::MoveGenerator,
    core::{Bitboard, Piece, BOARD_SIZE},
};
use lazy_static::lazy_static;
use rand::random;

use super::direction::{BISHOP_MOVES, ROOK_MOVES};

lazy_static! {
    pub static ref BISHOP_MAGICS: [Magic; BOARD_SIZE] = Magic::bishop();
    pub static ref ROOK_MAGICS: [Magic; BOARD_SIZE] = Magic::rook();
}

// TODO - save magic numbers and load them at program startup
// TODO - in most cases, blockers on the edge do not matter as the square will be attacked anyways, so mask them out when needed

pub struct Magic {
    table: Vec<Bitboard>, // lookup table to store the actual attacks on each square vs each blocker configuration
    mask: Bitboard,       // mask to remove non-important pieces
    number: Bitboard,     // number to multiply blockers by
    offset: usize,        // offset to shift by to get index
}

impl Magic {
    /// Fetches the attacked square bitboard, given an unmasked bitboard of blockers
    pub fn get(&self, blockers: Bitboard) -> Bitboard {
        // apply mask to the blockers first
        let masked_blockers = blockers & self.mask;

        let index = Self::calculate_index(masked_blockers, self.number, self.offset);
        self.table[index]
    }

    /// Generate bishop magic numbers
    pub fn bishop() -> [Magic; BOARD_SIZE] {
        // combine bishop attack directions into a single attack bitboard per square
        let mut bishop_attacks = [Bitboard::EMPTY; BOARD_SIZE];

        for (_, attack_ray) in BISHOP_MOVES.iter() {
            for square in 0..BOARD_SIZE {
                bishop_attacks[square] |= attack_ray[square];
            }
        }

        Self::generate_magics(Piece::Bishop, bishop_attacks)
    }

    /// Generate rook magic numbers
    pub fn rook() -> [Magic; BOARD_SIZE] {
        // combine rook attack directions into a single attack bitboard per square
        let mut rook_attacks = [Bitboard::EMPTY; BOARD_SIZE];

        for (_, attack_ray) in ROOK_MOVES.iter() {
            for square in 0..BOARD_SIZE {
                rook_attacks[square] |= attack_ray[square];
            }
        }

        Self::generate_magics(Piece::Rook, rook_attacks)
    }

    /// Helper function that does the magic number guess and check process
    fn generate_magics(
        piece: Piece,
        attacked_squares: [Bitboard; BOARD_SIZE],
    ) -> [Magic; BOARD_SIZE] {
        let mut square = 0;

        // generate a magic number for each attack board
        attacked_squares.map(|attack_board| {
            // TODO - added +1 for some leeway, having exact required amount of bits wasn't always working
            // total number of bits required to uniquely index each blocker square
            let blocker_count = attack_board.count_bits() + 1;

            // total number of possible permutations of blockers
            let permutations = 2usize.pow(blocker_count as u32);

            // storage for the magic number
            let mut magic_number;

            'generate: loop {
                // magic numbers are usually few in 1 bits, so AND a few random numbers together
                magic_number = Bitboard(random()) & Bitboard(random()) & Bitboard(random());

                // initialize lookup table with enough empty spaces and an enumerator of all blocker permutations
                let mut occupancy_table = vec![false; permutations];
                let mut blocker_subset = Bitboard::EMPTY;

                'check: loop {
                    // get next enumerated blocker configuration from attack board
                    blocker_subset = blocker_subset.next_subset(attack_board);

                    // calculate index and check if a value in the table is already using this index
                    let index = Self::calculate_index(blocker_subset, magic_number, blocker_count);

                    match occupancy_table[index] {
                        true => break 'check, // if already taken, this magic number won't work
                        false => occupancy_table[index] = true, // else, set this spot to taken and continue on
                    }

                    // if we have looped back to an empty board, then this magic number works
                    if blocker_subset.is_empty() {
                        break 'generate;
                    }
                }
            }

            // generate lookup table only once a valid magic number was found
            let mut blocker_subset = Bitboard::EMPTY;
            let mut lookup_table = vec![Bitboard::EMPTY; permutations];

            'map: loop {
                blocker_subset = blocker_subset.next_subset(attack_board);
                let index = Self::calculate_index(blocker_subset, magic_number, blocker_count);

                lookup_table[index] =
                    MoveGenerator::generate_sliding_attack(square, piece, blocker_subset);

                if blocker_subset.is_empty() {
                    break 'map;
                }
            }

            square += 1;

            // past here, we have the required info for the magic number
            Magic {
                table: lookup_table,
                mask: attack_board,
                number: magic_number,
                offset: blocker_count,
            }
        })
    }

    /// Helper function to do the index calculation for the lookup tables
    fn calculate_index(blockers: Bitboard, magic: Bitboard, offset: usize) -> usize {
        ((blockers * magic) >> (BOARD_SIZE - offset)).0 as usize
    }
}
