use crate::core::{Bitboard, BOARD_SIZE};
use rand::random;

use super::direction::{generate_sliding_attacks, Direction};

pub struct Magic {
    table: Vec<Option<Bitboard>>, // lookup table to store the actual attacks on each square vs each blocker configuration
    mask: Bitboard,               // mask to remove non-important pieces
    number: Bitboard,             // number to multiply blockers by
    offset: usize,                // offset to shift by to get index
}

impl Magic {
    pub fn get(&self, blockers: Bitboard) -> Bitboard {
        let masked_blockers = blockers & self.mask;
        let index = ((masked_blockers * self.number) >> (BOARD_SIZE - self.offset)).0 as usize;

        match self.table[index] {
            Some(attack) => attack,
            None => Bitboard::EMPTY,
        }
    }

    /// Generate rook magic numbers
    pub fn rook() -> [Magic; BOARD_SIZE] {
        // combine rook attack directions into a single attack bitboard per square
        let mut rook_attacks = [Bitboard::EMPTY; BOARD_SIZE];
        let attack_directions = Direction::STRAIGHTS.map(|dir| generate_sliding_attacks(dir));

        for attack_ray in attack_directions {
            for square in 0..BOARD_SIZE {
                rook_attacks[square] |= attack_ray[square];
            }
        }

        Self::generate_magics(rook_attacks)
    }

    /// Generate bishop magic numbers
    pub fn bishop() -> [Magic; BOARD_SIZE] {
        // combine bishop attack directions into a single attack bitboard per square
        let mut bishop_attacks = [Bitboard::EMPTY; BOARD_SIZE];
        let attack_directions = Direction::DIAGONALS.map(|dir| generate_sliding_attacks(dir));

        for attack_ray in attack_directions {
            for square in 0..BOARD_SIZE {
                bishop_attacks[square] |= attack_ray[square];
            }
        }

        Self::generate_magics(bishop_attacks)
    }

    /// Helper function that does the magic number guess and check process
    fn generate_magics(attacked_squares: [Bitboard; BOARD_SIZE]) -> [Magic; BOARD_SIZE] {
        let mut square = 0;

        // generate a magic number for each attack board
        let magics = attacked_squares.map(|attack_board| {
            // TODO - added +1 for some leeway, having exact required amount of bits wasn't working always
            // total number of bits required to uniquely index each blocker square
            let blocker_count = attack_board.count_bits() + 2;

            // total number of possible permutations of blockers
            let permutations = 2usize.pow(blocker_count as u32);

            // storage for the magic number
            let mut magic_number;
            let mut lookup_table: Vec<Option<Bitboard>>;

            'generate: loop {
                // magic numbers are usually few in 1 bits, so AND a few random numbers together
                magic_number = Bitboard(random()) & Bitboard(random()) & Bitboard(random());

                // initialize lookup table with enough empty spaces and an enumerator of all blocker permutations
                lookup_table = vec![None; permutations];
                let mut blocker_subset = Bitboard::EMPTY;

                'check: loop {
                    // bitwise trick to enumerate all subsets of attacker board
                    // https://www.chessprogramming.org/Traversing_Subsets_of_a_Set#All_Subsets_of_any_Set
                    blocker_subset = (blocker_subset - attack_board) & attack_board;

                    // calculate index and check if a value in the table is already using this index
                    let index = ((blocker_subset * magic_number) >> (BOARD_SIZE - blocker_count)).0
                        as usize;

                    match lookup_table[index] {
                        // if already taken, this magic number won't work
                        Some(_) => break 'check,

                        // else, set this spot to taken and continue on
                        // TODO - actually generate attacks at this spot based on blocker subset
                        None => lookup_table[index] = Some(blocker_subset),
                    }

                    // if we have looped back to an empty board, then this magic number works
                    if blocker_subset.is_empty() {
                        break 'generate;
                    }
                }
            }

            println!(
                "found magic number {} at offset {} for square {}",
                magic_number.0, blocker_count, square
            );

            square += 1;

            // past here, we have the required info for the magic number
            Magic {
                table: lookup_table,
                mask: attack_board,
                number: magic_number,
                offset: blocker_count,
            }
        });

        println!("finished");

        magics
    }
}
