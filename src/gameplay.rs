use rand::prelude::*;
use std::{
    fmt::{self, Display, Formatter},
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum DisplayMode {
    Ascii,
    Unicode,
    ColoredEmoji,
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
pub struct Card(pub(crate) Value, pub(crate) Suit);

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
                    DisplayMode::Unicode => "♠",
                    DisplayMode::ColoredEmoji => "♠️",
                },
                Suit::Hearts => match self.mode {
                    DisplayMode::Ascii => "h",
                    DisplayMode::Unicode => "♥",
                    DisplayMode::ColoredEmoji => "♥️",
                },
                Suit::Diamonds => match self.mode {
                    DisplayMode::Ascii => "d",
                    DisplayMode::Unicode => "♦",
                    DisplayMode::ColoredEmoji => "♦️",
                },
                Suit::Clubs => match self.mode {
                    DisplayMode::Ascii => "c",
                    DisplayMode::Unicode => "♣",
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
                "{}{}",
                self.card.value(),
                self.card.suit().display(self.mode)
            )
        }
    }
}

pub mod headsup;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Deck([Card; 52]);

impl Default for Deck {
    fn default() -> Self {
        let mut cards = [Default::default(); 52];
        let values = [
            Value::Deuce,
            Value::Trey,
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
            Value::Ace,
        ];
        let suits = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

        for (i, &value) in values.iter().enumerate() {
            for (j, &suit) in suits.iter().enumerate() {
                cards[i * 4 + j] = Card(value, suit);
            }
        }

        Self(cards)
    }
}

use std::array::IntoIter;

impl Deck {
    pub fn shuffle(&mut self) {
        self.0.shuffle(&mut rand::rng());
    }

    pub fn shuffled(&self) -> Self {
        let mut deck = *self;
        deck.shuffle();
        deck
    }

    pub fn deal(&self) -> IntoIter<Card, 52> {
        self.0.into_iter()
    }
}
