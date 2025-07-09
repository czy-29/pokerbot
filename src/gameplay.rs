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
        if s.len() != 2 || !s.is_ascii() {
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
    // Should check the length of the slice before calling this
    fn from_slice(cards: &[Card]) -> Self {
        Self(cards.try_into().unwrap())
    }

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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum ParserResult<T> {
    Err,
    None,
    OkSome(T),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct CardsParser<'a>(&'a str);

impl<'a> CardsParser<'a> {
    fn card_eaten(&self) -> ParserResult<(Card, Self)> {
        let s = self.0.trim();
        if s.is_empty() {
            return ParserResult::None;
        }
        if s.len() == 1 {
            return ParserResult::Err;
        }
        match Card::from_str(&s[0..2]) {
            Ok(card) => ParserResult::OkSome((card, Self(&s[2..]))),
            Err(_) => ParserResult::Err,
        }
    }

    fn eat_card(&mut self) -> ParserResult<Card> {
        match self.card_eaten() {
            ParserResult::OkSome((card, next)) => {
                self.0 = next.0;
                ParserResult::OkSome(card)
            }
            ParserResult::None => ParserResult::None,
            ParserResult::Err => ParserResult::Err,
        }
    }

    fn eat_cards<const N: usize>(&mut self) -> Option<CardsCombined<N>> {
        let mut cards = [Card::default(); N];
        let mut parser = *self;
        for i in 0..N {
            match parser.card_eaten() {
                ParserResult::OkSome((card, next)) => {
                    cards[i] = card;
                    parser = next;
                }
                _ => return None,
            }
        }
        let cards = CardsCombined::new(cards);
        if cards.is_some() {
            self.0 = parser.0;
        }
        cards
    }
}

impl<const N: usize> FromStr for CardsCombined<N> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err(());
        }
        let mut parser = CardsParser(s);
        match parser.eat_cards::<N>() {
            Some(cards) => {
                if parser.0.is_empty() {
                    Ok(cards)
                } else {
                    Err(())
                }
            }
            None => Err(()),
        }
    }
}

pub type Hole = CardsCombined<2>;
pub type Flop = CardsCombined<3>;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Board(BoardCards);

impl Deref for Board {
    type Target = BoardCards;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Board {
    pub fn from_slice(cards: &[Card]) -> Option<Self> {
        if cards.is_empty() {
            return Some(Default::default());
        }

        if !(3..=5).contains(&cards.len()) {
            return None; // Invalid number of cards for a board
        }

        if cards.has_dup() {
            return None; // Cannot have duplicate cards
        }

        let flop = Flop::from_slice(&cards[0..3]);
        match cards.len() {
            3 => Some(Self(BoardCards::Flop(flop))),
            4 => Some(Self(BoardCards::Turn {
                flop,
                turn: cards[3],
            })),
            5 => Some(Self(BoardCards::River {
                flop,
                turn: cards[3],
                river: cards[4],
            })),
            _ => unreachable!(), // Since we checked the length above
        }
    }

    pub fn to_vec(&self) -> Vec<Card> {
        match self.0 {
            BoardCards::Preflop => vec![],
            BoardCards::Flop(flop) => flop.into_iter().collect(),
            BoardCards::Turn { flop, turn } => {
                let mut cards = flop.into_iter().collect::<Vec<_>>();
                cards.push(turn);
                cards
            }
            BoardCards::River { flop, turn, river } => {
                let mut cards = flop.into_iter().collect::<Vec<_>>();
                cards.push(turn);
                cards.push(river);
                cards
            }
        }
    }

    pub fn flop(flop: Flop) -> Self {
        Self(BoardCards::Flop(flop))
    }

    pub fn turn(&self, turn: Card) -> Option<Self> {
        if let BoardCards::Flop(flop) = self.0 {
            if flop.contains(&turn) {
                None // Cannot have duplicate cards
            } else {
                Some(Self(BoardCards::Turn { flop, turn }))
            }
        } else {
            None
        }
    }

    pub fn river(&self, river: Card) -> Option<Self> {
        if let BoardCards::Turn { flop, turn } = self.0 {
            if flop.contains(&river) || turn == river {
                None // Cannot have duplicate cards
            } else {
                Some(Self(BoardCards::River { flop, turn, river }))
            }
        } else {
            None
        }
    }

    pub fn display(self, mode: DisplayMode) -> BoardDisplay {
        BoardDisplay { board: self, mode }
    }
}

impl FromStr for Board {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err(());
        }

        let s = s.trim();
        if s == "x" {
            return Ok(Self::default());
        }

        let mut parser = CardsParser(s);
        match parser.eat_cards::<3>() {
            Some(flop) => {
                let board = Self::flop(flop);
                match parser.eat_card() {
                    ParserResult::OkSome(turn) => match board.turn(turn) {
                        Some(board) => match parser.eat_card() {
                            ParserResult::OkSome(river) => match board.river(river) {
                                Some(board) => {
                                    if parser.0.is_empty() {
                                        Ok(board) // River board
                                    } else {
                                        Err(())
                                    }
                                }
                                None => Err(()),
                            },
                            ParserResult::None => Ok(board), // Turn board
                            ParserResult::Err => Err(()),
                        },
                        None => Err(()),
                    },
                    ParserResult::None => Ok(board), // Flop board
                    ParserResult::Err => Err(()),
                }
            }
            None => Err(()),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum BoardCards {
    #[default]
    Preflop,
    Flop(Flop),
    Turn {
        flop: Flop,
        turn: Card,
    },
    River {
        flop: Flop,
        turn: Card,
        river: Card,
    },
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

    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct BoardDisplay {
        pub(super) board: Board,
        pub(super) mode: DisplayMode,
    }

    impl Display for BoardDisplay {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let delimiter = match self.mode {
                DisplayMode::Ascii => "  ",
                DisplayMode::Unicode | DisplayMode::ColoredUnicode => "   ",
                DisplayMode::ColoredEmoji => "    ",
            };
            match self.board.0 {
                BoardCards::Preflop => write!(f, "x"),
                BoardCards::Flop(flop) => write!(f, "{}", flop.display(self.mode)),
                BoardCards::Turn { flop, turn } => {
                    write!(
                        f,
                        "{}{}{}",
                        flop.display(self.mode),
                        delimiter,
                        turn.display(self.mode)
                    )
                }
                BoardCards::River { flop, turn, river } => {
                    write!(
                        f,
                        "{}{}{}{}{}",
                        flop.display(self.mode),
                        delimiter,
                        turn.display(self.mode),
                        delimiter,
                        river.display(self.mode),
                    )
                }
            }
        }
    }
}

pub mod headsup;
