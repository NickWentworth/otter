pub struct Direction;
impl Direction {
    pub const N: isize = -8;
    pub const E: isize = 1;
    pub const S: isize = 8;
    pub const W: isize = -1;
    pub const NE: isize = Self::N + Self::E;
    pub const NW: isize = Self::N + Self::W;
    pub const SE: isize = Self::S + Self::E;
    pub const SW: isize = Self::S + Self::W;

    pub const DIAGONALS: [isize; 4] = [Self::NE, Self::NW, Self::SE, Self::SW];
    pub const STRAIGHTS: [isize; 4] = [Self::N, Self::E, Self::S, Self::W];
}
