use est::slice::SliceExt;
use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    str::FromStr,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Value {
    Deuce,
    Trey,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Self::Deuce => '2',
            Self::Trey => '3',
            Self::Four => '4',
            Self::Five => '5',
            Self::Six => '6',
            Self::Seven => '7',
            Self::Eight => '8',
            Self::Nine => '9',
            Self::Ten => 'T',
            Self::Jack => 'J',
            Self::Queen => 'Q',
            Self::King => 'K',
            Self::Ace => 'A',
        };
        write!(f, "{}", symbol)
    }
}

impl FromStr for Value {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "2" => Ok(Self::Deuce),
            "3" => Ok(Self::Trey),
            "4" => Ok(Self::Four),
            "5" => Ok(Self::Five),
            "6" => Ok(Self::Six),
            "7" => Ok(Self::Seven),
            "8" => Ok(Self::Eight),
            "9" => Ok(Self::Nine),
            "T" => Ok(Self::Ten),
            "J" => Ok(Self::Jack),
            "Q" => Ok(Self::Queen),
            "K" => Ok(Self::King),
            "A" => Ok(Self::Ace),
            _ => Err(()),
        }
    }
}

impl Value {
    fn as_u8(self) -> u8 {
        match self {
            Self::Deuce => 0,
            Self::Trey => 1,
            Self::Four => 2,
            Self::Five => 3,
            Self::Six => 4,
            Self::Seven => 5,
            Self::Eight => 6,
            Self::Nine => 7,
            Self::Ten => 8,
            Self::Jack => 9,
            Self::Queen => 10,
            Self::King => 11,
            Self::Ace => 12,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum DisplayMode {
    Ascii,
    Unicode,
    ColoredUnicode,
    ColoredEmoji,
}

impl DisplayMode {
    pub fn is_unicode(self) -> bool {
        matches!(self, Self::Unicode | Self::ColoredUnicode)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

use display::*;

impl Suit {
    pub fn display(self, mode: DisplayMode) -> SuitDisplay {
        SuitDisplay { suit: self, mode }
    }

    fn as_u8(self) -> u8 {
        match self {
            Self::Spades => 0,
            Self::Hearts => 1,
            Self::Diamonds => 2,
            Self::Clubs => 3,
        }
    }
}

impl FromStr for Suit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "s" => Ok(Self::Spades),
            "h" => Ok(Self::Hearts),
            "d" => Ok(Self::Diamonds),
            "c" => Ok(Self::Clubs),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Card(Value, Suit);

impl Default for Card {
    fn default() -> Self {
        Self(Value::Ace, Suit::Spades)
    }
}

impl Card {
    pub fn new(value: Value, suit: Suit) -> Self {
        Self(value, suit)
    }

    pub fn value(&self) -> Value {
        self.0
    }

    pub fn suit(&self) -> Suit {
        self.1
    }

    pub fn display(self, mode: DisplayMode) -> CardDisplay {
        CardDisplay { card: self, mode }
    }

    fn as_u8(self) -> u8 {
        (self.value().as_u8() << 2) | self.suit().as_u8()
    }
}

impl FromStr for Card {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(());
        }
        let value = Value::from_str(&s[0..1])?;
        let suit = Suit::from_str(&s[1..2])?;
        Ok(Self(value, suit))
    }
}

#[derive(Debug, Eq, Clone, Copy, Hash)]
pub struct CardsCombined<const N: usize>([Card; N]);

impl<const N: usize> PartialEq for CardsCombined<N> {
    fn eq(&self, other: &Self) -> bool {
        self.sorted() == other.sorted()
    }
}

impl<const N: usize> Deref for CardsCombined<N> {
    type Target = [Card; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> CardsCombined<N> {
    fn sorted(&self) -> [Card; N] {
        let mut sorted = self.0;
        sorted.sort_by(|a, b| a.as_u8().cmp(&b.as_u8()));
        sorted
    }

    pub fn new(cards: [Card; N]) -> Option<Self> {
        if cards.has_dup() {
            None // Cannot have duplicate cards
        } else {
            Some(Self(cards))
        }
    }

    pub fn display(self, mode: DisplayMode) -> CardsDisplay<N> {
        CardsDisplay { cards: self, mode }
    }
}

pub type Hole = CardsCombined<2>;
pub type Flop = CardsCombined<3>;

pub mod display {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct SuitDisplay {
        pub(super) suit: Suit,
        pub(super) mode: DisplayMode,
    }

    impl Display for SuitDisplay {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let str = match self.suit {
                Suit::Spades => match self.mode {
                    DisplayMode::Ascii => "s",
                    DisplayMode::Unicode | DisplayMode::ColoredUnicode => "♠",
                    DisplayMode::ColoredEmoji => "♠️",
                },
                Suit::Hearts => match self.mode {
                    DisplayMode::Ascii => "h",
                    DisplayMode::Unicode => "♥",
                    DisplayMode::ColoredUnicode => "\x1b[91m♥\x1b[0m",
                    DisplayMode::ColoredEmoji => "♥️",
                },
                Suit::Diamonds => match self.mode {
                    DisplayMode::Ascii => "d",
                    DisplayMode::Unicode => "♦",
                    DisplayMode::ColoredUnicode => "\x1b[91m♦\x1b[0m",
                    DisplayMode::ColoredEmoji => "♦️",
                },
                Suit::Clubs => match self.mode {
                    DisplayMode::Ascii => "c",
                    DisplayMode::Unicode | DisplayMode::ColoredUnicode => "♣",
                    DisplayMode::ColoredEmoji => "♣️",
                },
            };
            write!(f, "{}", str)
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct CardDisplay {
        pub(super) card: Card,
        pub(super) mode: DisplayMode,
    }

    impl Display for CardDisplay {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}{}{}",
                self.card.value(),
                if self.mode.is_unicode() { " " } else { "" },
                self.card.suit().display(self.mode)
            )
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct CardsDisplay<const N: usize> {
        pub(super) cards: CardsCombined<N>,
        pub(super) mode: DisplayMode,
    }

    impl<const N: usize> Display for CardsDisplay<N> {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let delimiter = if self.mode == DisplayMode::Ascii {
                " "
            } else {
                "  "
            };
            for (i, card) in self.cards.iter().enumerate() {
                if i > 0 {
                    write!(f, "{}", delimiter)?;
                }
                write!(f, "{}", card.display(self.mode))?;
            }
            Ok(())
        }
    }
}

impl<const N: usize> FromStr for CardsCombined<N> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Trim the input string
        let s = s.trim();
        
        // Split the input string by whitespace
        let card_strs: Vec<&str> = s.split_whitespace().collect();
        
        // If we have exactly one part and its length is N*2, try to parse as concatenated cards
        if card_strs.len() == 1 && card_strs[0].len() == N * 2 {
            let s = card_strs[0];
            
            // Try to parse as concatenated cards
            let mut cards = [Card::default(); N];
            for i in 0..N {
                let start = i * 2;
                let end = start + 2;
                if end <= s.len() {
                    let card_str = &s[start..end];
                    cards[i] = Card::from_str(card_str)?;
                } else {
                    return Err(());
                }
            }
            return CardsCombined::new(cards).ok_or(());
        }
        
        // Check if the number of cards matches N
        if card_strs.len() != N {
            return Err(());
        }
        
        // Parse each card string
        let mut cards = [Card::default(); N];
        for (i, card_str) in card_strs.iter().enumerate() {
            cards[i] = Card::from_str(card_str)?;
        }
        
        // Create a new CardsCombined, checking for duplicates
        CardsCombined::new(cards).ok_or(())
    }
}

pub mod headsup;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cards_combined_from_str() {
        // Test valid inputs with different spacing
        let hole1: Result<Hole, _> = "As Kc".parse();
        assert!(hole1.is_ok());
        
        let hole2: Result<Hole, _> = "As  Kc".parse();
        assert!(hole2.is_ok());
        
        let hole3: Result<Hole, _> = "AsKc".parse();
        assert!(hole3.is_ok());
        
        let hole4: Result<Hole, _> = " As Kc ".parse();
        assert!(hole4.is_ok());
        
        // Test invalid inputs
        
        // Wrong number of cards
        let invalid1: Result<Hole, _> = "As".parse();
        assert!(invalid1.is_err());
        
        let invalid2: Result<Hole, _> = "As Kc Qd".parse();
        assert!(invalid2.is_err());
        
        // Invalid card format
        let invalid3: Result<Hole, _> = "A s K c".parse();
        assert!(invalid3.is_err());
        
        // Duplicate cards
        let invalid4: Result<Hole, _> = "As As".parse();
        assert!(invalid4.is_err());
        
        // Test with different N values
        let flop: Result<Flop, _> = "As Kc Qd".parse();
        assert!(flop.is_ok());
        
        // Custom size
        type FourCards = CardsCombined<4>;
        let four_cards: Result<FourCards, _> = "As Kc Qd Jh".parse();
        assert!(four_cards.is_ok());
    }
}
