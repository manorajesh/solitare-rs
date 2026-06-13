// definitions ---

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Default)]
pub enum Suit {
    #[default]
    Hearts = 0,
    Diamonds = 1,
    Clubs = 2,
    Spades = 3,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Default)]
pub enum Value {
    #[default]
    Ace = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Default)]
pub struct Card {
    pub value: Value,
    pub suit: Suit,
    pub face_up: bool,
}

// impls ---

impl std::fmt::Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Suit::Hearts => write!(f, "♥"),
            Suit::Diamonds => write!(f, "♦"),
            Suit::Clubs => write!(f, "♣"),
            Suit::Spades => write!(f, "♠"),
        }
    }
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
}

impl Into<usize> for Suit {
    fn into(self) -> usize {
        // TODO: not safe
        self as usize
    }
}

impl From<usize> for Suit {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Hearts,
            1 => Self::Diamonds,
            2 => Self::Clubs,
            3 => Self::Spades,
            _ => Self::Hearts
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Ace => write!(f, "A"),
            Value::Two => write!(f, "2"),
            Value::Three => write!(f, "3"),
            Value::Four => write!(f, "4"),
            Value::Five => write!(f, "5"),
            Value::Six => write!(f, "6"),
            Value::Seven => write!(f, "7"),
            Value::Eight => write!(f, "8"),
            Value::Nine => write!(f, "9"),
            Value::Ten => write!(f, "10"),
            Value::Jack => write!(f, "J"),
            Value::Queen => write!(f, "Q"),
            Value::King => write!(f, "K"),
        }
    }
}

impl Value {
    pub const ALL: [Value; 13] = [
        Value::Ace,
        Value::Two,
        Value::Three,
        Value::Four,
        Value::Five,
        Value::Six,
        Value::Seven,
        Value::Eight,
        Value::Nine,
        Value::Ten,
        Value::Jack,
        Value::Queen,
        Value::King,
    ];
}

impl Card {
    pub fn new(value: Value, suit: Suit, face_up: bool) -> Self {
        Card { value, suit, face_up }
    }
}
