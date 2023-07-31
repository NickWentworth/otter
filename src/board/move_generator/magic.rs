use super::direction::{BISHOP_RAYS, ROOK_RAYS};
use crate::{
    board::move_generator::MoveGenerator,
    core::{Bitboard, Piece, Square, BOARD_SIZE},
};
use lazy_static::lazy_static;
use rand::random;

lazy_static! {
    pub static ref BISHOP_MAGICS: [Magic; BOARD_SIZE] = Magic::load_bishop_magics();
    pub static ref ROOK_MAGICS: [Magic; BOARD_SIZE] = Magic::load_rook_magics();
}

// TODO - in most cases, blockers on the edge do not matter as the square will be attacked anyways, so mask them out when needed

pub struct Magic {
    table: Vec<Bitboard>, // lookup table to store the actual attacks on each square vs each blocker configuration
    mask: Bitboard,       // mask to remove non-important pieces
    number: Bitboard,     // number to multiply blockers by
    offset: usize,        // offset to shift by to get index
}

impl Magic {
    /// How many extra bits are given to the offset for easier number generation
    const LENIENCY_BITS: usize = 1;

    /// Fetches the attacked square bitboard, given an unmasked bitboard of blockers
    pub fn get(&self, blockers: Bitboard) -> Bitboard {
        // apply mask to the blockers first
        let masked_blockers = blockers & self.mask;

        let index = Self::calculate_index(masked_blockers, self.number, self.offset);
        self.table[index]
    }

    /// Loads an array of bishop magics from pre-generated numbers
    pub fn load_bishop_magics() -> [Magic; BOARD_SIZE] {
        let mut square = 0;

        Self::fold_bishop_attacks().map(|attack_board| {
            let magic = Self::load_single_magic(
                attack_board,
                Self::BISHOP_NUMBERS[square],
                square,
                Piece::Bishop,
            );

            square += 1;
            magic
        })
    }

    /// Loads an array of rook magics from pre-generated numbers
    pub fn load_rook_magics() -> [Magic; BOARD_SIZE] {
        let mut square = 0;

        Self::fold_rook_attacks().map(|attack_board| {
            let magic = Self::load_single_magic(
                attack_board,
                Self::ROOK_NUMBERS[square],
                square,
                Piece::Rook,
            );

            square += 1;
            magic
        })
    }

    /// Generates the magic struct given a blockers board and a pre-generated magic number
    fn load_single_magic(
        blockers: Bitboard,
        number: Bitboard,
        square: Square,
        piece: Piece,
    ) -> Magic {
        // get blocker count and permutations
        let blocker_count = blockers.count_bits() + Self::LENIENCY_BITS;
        let table_size = 2usize.pow(blocker_count as u32);

        let mut lookup_table = vec![Bitboard::EMPTY; table_size];

        // fill table with attacks based on blocker permutations
        let mut blocker_subset = Bitboard::EMPTY;
        loop {
            blocker_subset = blocker_subset.next_subset(blockers);
            let index = Self::calculate_index(blocker_subset, number, blocker_count);

            lookup_table[index] =
                MoveGenerator::generate_sliding_attack(square, piece, blocker_subset);

            if blocker_subset.is_empty() {
                break;
            }
        }

        Magic {
            table: lookup_table,
            mask: blockers,
            number,
            offset: blocker_count,
        }
    }

    /// Generates rook and bishop magic numbers and prints them, where they should then be copied into this file
    ///
    /// Only needs to be run once to actually generate the numbers, afterwards the numbers are assumed to be valid
    /// and should not be touched unless re-generated from here
    pub fn generate_magics() {
        // combine sliding piece attack directions into a single attack bitboard per square
        let mut bishop_attacks = Self::fold_bishop_attacks();
        let mut rook_attacks = Self::fold_rook_attacks();

        // now do magic number generation
        for square in 0..BOARD_SIZE {
            // we can just overwrite the blockers array with the magic number
            bishop_attacks[square] = Self::generate_single_magic(bishop_attacks[square]);
            rook_attacks[square] = Self::generate_single_magic(rook_attacks[square]);
        }

        // finally print out the generated magic numbers
        println!("Bishop numbers: {:?}", bishop_attacks);
        println!();
        println!("Rook numbers: {:?}", rook_attacks);
    }

    /// Helper function to generate one magic number
    fn generate_single_magic(blockers: Bitboard) -> Bitboard {
        let blocker_count = blockers.count_bits() + Self::LENIENCY_BITS;
        let table_size = 2usize.pow(blocker_count as u32);

        loop {
            // genrate a magic number candidate
            // generally good to have few 1-bits in the number, so AND a few random values together
            let magic_number = Bitboard(random()) & Bitboard(random()) & Bitboard(random());

            // create an occupancy table and enumerate over all possible blocker permutations
            let mut blocker_subset = Bitboard::EMPTY;
            let mut occupancy_table = vec![false; table_size];

            'check: loop {
                // get next enumerated blocker configuration from attack board
                blocker_subset = blocker_subset.next_subset(blockers);

                // calculate index and check if a value in the table is already using this index
                let index = Self::calculate_index(blocker_subset, magic_number, blocker_count);

                match occupancy_table[index] {
                    true => break 'check, // if already taken, this magic number won't work
                    false => occupancy_table[index] = true, // else, set this spot to taken and continue on
                }

                // if we have looped back to an empty board, then this magic number works
                if blocker_subset.is_empty() {
                    return magic_number;
                }
            }
        }
    }

    /// Helper function to do the index calculation for the lookup tables
    fn calculate_index(blockers: Bitboard, magic: Bitboard, offset: usize) -> usize {
        ((blockers * magic) >> (BOARD_SIZE - offset)).0 as usize
    }

    /// Helper function to fold all bishop attack rays into single attack bitboard
    fn fold_bishop_attacks() -> [Bitboard; BOARD_SIZE] {
        let mut bishop_attacks = [Bitboard::EMPTY; BOARD_SIZE];

        for (_, attack_ray) in BISHOP_RAYS.iter() {
            for square in 0..BOARD_SIZE {
                bishop_attacks[square] |= attack_ray[square];
            }
        }

        bishop_attacks
    }

    /// Helper function to fold all rook attack rays into single attack bitboard
    fn fold_rook_attacks() -> [Bitboard; BOARD_SIZE] {
        let mut rook_attacks = [Bitboard::EMPTY; BOARD_SIZE];

        for (_, attack_ray) in ROOK_RAYS.iter() {
            for square in 0..BOARD_SIZE {
                rook_attacks[square] |= attack_ray[square];
            }
        }

        rook_attacks
    }

    // pre-generated magic numbers to be used for each square
    // much faster to generate good ones first and get lookup tables after
    const BISHOP_NUMBERS: [Bitboard; BOARD_SIZE] = [
        Bitboard(297950063305457672),
        Bitboard(2341876238643364385),
        Bitboard(2884946996135731330),
        Bitboard(865394893475860608),
        Bitboard(9223460552641347876),
        Bitboard(72057636998094884),
        Bitboard(9295500069027463680),
        Bitboard(9547648876316475400),
        Bitboard(1153204364987400218),
        Bitboard(18166277163978761),
        Bitboard(38883131629043730),
        Bitboard(9529757618998943745),
        Bitboard(72058762302620680),
        Bitboard(1730517038843047969),
        Bitboard(434667750311329798),
        Bitboard(612498346757914658),
        Bitboard(38281150901272580),
        Bitboard(2891311116465471556),
        Bitboard(865834625916993601),
        Bitboard(4758071732407898146),
        Bitboard(857632088789026),
        Bitboard(360853187902833160),
        Bitboard(5765770394288529409),
        Bitboard(36592026214465712),
        Bitboard(72128523275339784),
        Bitboard(4504222431314946),
        Bitboard(4683779348744897540),
        Bitboard(13911636843281027845),
        Bitboard(9370055685166866466),
        Bitboard(649081365082705924),
        Bitboard(5766859494646874260),
        Bitboard(9228440824654729410),
        Bitboard(5548452883516883032),
        Bitboard(576742519344333057),
        Bitboard(577076483110469697),
        Bitboard(22518066925535264),
        Bitboard(9232383634156356609),
        Bitboard(36081574130762243),
        Bitboard(1265529091805954084),
        Bitboard(9251099658590617793),
        Bitboard(586136454695069700),
        Bitboard(72629340353069344),
        Bitboard(11547299813608390784),
        Bitboard(613061331876579328),
        Bitboard(74309402575913088),
        Bitboard(13871227590058115073),
        Bitboard(4680759529177622),
        Bitboard(1154047413170884625),
        Bitboard(4785353785891841),
        Bitboard(4503608225701904),
        Bitboard(180144053881421832),
        Bitboard(9295429700686070784),
        Bitboard(9016272373223424),
        Bitboard(1207036444644741124),
        Bitboard(1157461426806198273),
        Bitboard(1152991875501589537),
        Bitboard(9520647030083682564),
        Bitboard(2307050307344925184),
        Bitboard(12106819308789700627),
        Bitboard(1730121689218679296),
        Bitboard(1297340164409917952),
        Bitboard(172403492492219392),
        Bitboard(9840365326531823680),
        Bitboard(4521211417604417),
    ];
    const ROOK_NUMBERS: [Bitboard; BOARD_SIZE] = [
        Bitboard(12105712082324340754),
        Bitboard(9583660885365883427),
        Bitboard(180707625472884753),
        Bitboard(72236814622148737),
        Bitboard(73326447657418769),
        Bitboard(3378799770796163),
        Bitboard(72093053423191297),
        Bitboard(4935953996349196545),
        Bitboard(38333390797152257),
        Bitboard(9871892032497125634),
        Bitboard(18931391216485380),
        Bitboard(9008745476522056),
        Bitboard(1152922604252725448),
        Bitboard(76583270879461648),
        Bitboard(583286529348160576),
        Bitboard(2535474956603432),
        Bitboard(581387389029844993),
        Bitboard(4611697634170642433),
        Bitboard(2305878761596452865),
        Bitboard(108231530886791176),
        Bitboard(9737909398222733328),
        Bitboard(16753462116434313248),
        Bitboard(2305845225418916104),
        Bitboard(9223374237021241536),
        Bitboard(5850176139563106345),
        Bitboard(140739925247009),
        Bitboard(4648295634091114627),
        Bitboard(2251937290911746),
        Bitboard(590534518319087824),
        Bitboard(2253586520408080),
        Bitboard(18049582965460993),
        Bitboard(1234127035421433870),
        Bitboard(6666460334081),
        Bitboard(1747964012076793890),
        Bitboard(1442277849418440706),
        Bitboard(282025806528514),
        Bitboard(4785075946291216),
        Bitboard(4508006297894944),
        Bitboard(4611704727313334276),
        Bitboard(9223376435203293698),
        Bitboard(1144179288705541),
        Bitboard(2308236164991680546),
        Bitboard(10520550566541660197),
        Bitboard(282574622556292),
        Bitboard(1161946914523971653),
        Bitboard(2305882643172434056),
        Bitboard(281492425547840),
        Bitboard(4611690624779813392),
        Bitboard(2306476328449757317),
        Bitboard(2253998837989409),
        Bitboard(72341268071514180),
        Bitboard(144123984173597731),
        Bitboard(3055696748439142404),
        Bitboard(437130641013212437),
        Bitboard(1734457603193832449),
        Bitboard(4611726150602063888),
        Bitboard(9871890660248814617),
        Bitboard(4503642845520130),
        Bitboard(1297045489054908674),
        Bitboard(288247968875284484),
        Bitboard(2597451119513157648),
        Bitboard(1161928706080514065),
        Bitboard(298363492501618752),
        Bitboard(1164180650241296512),
    ];
}
