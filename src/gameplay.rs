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

pub mod headsup;
