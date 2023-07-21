use crate::core::{Bitboard, BOARD_SIZE};
use rand::random;

use super::direction::{generate_sliding_attacks, Direction};

pub struct Magic {
    mask: Bitboard,   // mask to remove non-important pieces
    number: Bitboard, // number to multiply blockers by
    offset: usize,    // offset to shift by to get index
}

pub fn generate_rook_magics() -> [Magic; BOARD_SIZE] {
    // combine rook attack directions into a single attack bitboard per square
    let mut rook_attacks = [Bitboard::EMPTY; BOARD_SIZE];
    for direction in Direction::STRAIGHTS {
        let attack_table = generate_sliding_attacks(direction);

        for square in 0..BOARD_SIZE {
            rook_attacks[square] |= attack_table[square];
        }
    }

    // generate a magic number for each attack board
    let magics = rook_attacks.map(|attack_board| {
        // TODO - added +1 for some leeway, having exact required amount of bits wasn't working always
        // total number of bits required to uniquely index each blocker square
        let blocker_count = attack_board.count_bits();

        // total number of possible permutations of blockers
        let permutations = 2usize.pow(blocker_count as u32);

        // storage for the magic number
        let mut magic_number;

        'generate: loop {
            // magic numbers are usually few in 1 bits, so AND a few random numbers together
            magic_number = Bitboard(random()) & Bitboard(random()) & Bitboard(random());

            // initialize lookup table with enough empty spaces and an enumerator of all blocker permutations
            let mut lookup_table = vec![false; permutations];
            let mut blocker_subset = Bitboard::EMPTY;

            'check: loop {
                // bitwise trick to enumerate all subsets of attacker board
                // https://www.chessprogramming.org/Traversing_Subsets_of_a_Set#All_Subsets_of_any_Set
                blocker_subset = (blocker_subset - attack_board) & attack_board;

                // calculate index and check if a value in the table is already using this index
                let index =
                    ((blocker_subset * magic_number) >> (BOARD_SIZE - blocker_count)).0 as usize;

                match lookup_table[index] {
                    true => break 'check, // if already taken, this magic number won't work
                    false => lookup_table[index] = true, // else, set this spot to taken and continue on
                }

                // if we have looped back to an empty board, then this magic number works
                if blocker_subset.is_empty() {
                    break 'generate;
                }
            }
        }

        println!(
            "found magic number {} at offset {}",
            magic_number.0, blocker_count
        );

        // past here, we have the required info for the magic number
        Magic {
            mask: attack_board,
            number: magic_number,
            offset: blocker_count,
        }
    });

    println!("finished");

    magics
}
