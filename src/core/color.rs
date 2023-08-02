#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Returns the opposite color to the given one
    pub fn opposite(self) -> Color {
        use Color::*;

        match self {
            White => Black,
            Black => White,
        }
    }

    /// Converts an ascii character and a color to the correct casing
    ///
    /// Requires an initial character, so `From<Color>` cannot be used
    pub fn to_char(self, c: char) -> char {
        use Color::*;

        match self {
            White => c.to_ascii_uppercase(),
            Black => c.to_ascii_lowercase(),
        }
    }
}

impl From<char> for Color {
    /// Converts an ascii character to a color
    fn from(value: char) -> Self {
        use Color::*;

        match value.is_ascii_uppercase() {
            true => White,
            false => Black,
        }
    }
}

pub const NUM_COLORS: usize = 2;
