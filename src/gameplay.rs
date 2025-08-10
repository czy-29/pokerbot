use itertools::Itertools;
use rayon::prelude::*;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
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

    fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Deuce,
            1 => Self::Trey,
            2 => Self::Four,
            3 => Self::Five,
            4 => Self::Six,
            5 => Self::Seven,
            6 => Self::Eight,
            7 => Self::Nine,
            8 => Self::Ten,
            9 => Self::Jack,
            10 => Self::Queen,
            11 => Self::King,
            12 => Self::Ace,
            _ => unreachable!(),
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

    fn is_red(self) -> bool {
        matches!(self.suit(), Suit::Hearts | Suit::Diamonds)
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
    fn from_slice(cards: &[Card]) -> Self {
        Self(
            cards
                .try_into()
                .expect("Should check the length of the slice before calling this"),
        )
    }

    fn unchecked(cards: [Card; N]) -> Self {
        Self(cards)
    }

    fn sorted(&self) -> [Card; N] {
        let mut sorted = self.0;
        sorted.sort_by(|a, b| a.as_u8().cmp(&b.as_u8()));
        sorted
    }

    pub fn new(cards: [Card; N]) -> Option<Self> {
        if !cards.iter().all_unique() {
            None // Cannot have duplicate cards
        } else {
            Some(Self(cards))
        }
    }

    pub fn display(self, mode: DisplayMode) -> CardsDisplay<N> {
        CardsDisplay { cards: self, mode }
    }

    fn is_flush(&self) -> bool {
        self.0.iter().map(Card::suit).all_equal()
    }

    fn to_sorted_values(&self) -> [Value; N] {
        let mut values = self.0.map(|card| card.value());
        values.sort_unstable_by(|a, b| b.cmp(a));
        values
    }

    fn check_straight(mut u8s: [u8; N]) -> Option<Value> {
        u8s.sort_unstable();

        if u8s.windows(2).all(|w| w[1] == w[0] + 1) {
            Some(Value::from_u8(u8s[N - 1] - 1))
        } else {
            None
        }
    }

    fn is_straight(&self) -> Option<Value> {
        const ACE: u8 = 13;
        let mut u8s = self.0.map(|card| card.value().as_u8() + 1);
        let check_straight = Self::check_straight(u8s);

        if check_straight.is_none() && u8s.contains(&ACE) {
            // Check for wheel (A-2-3-4-5)
            for u in &mut u8s {
                if *u == ACE {
                    *u = 0;
                    break;
                }
            }

            return Self::check_straight(u8s);
        }

        check_straight
    }

    fn to_value_map(&self) -> ValueMap {
        let mut value_map: BTreeMap<usize, BTreeSet<Value>> = BTreeMap::new();

        for (value, count) in self.0.iter().map(Card::value).counts() {
            value_map
                .entry(count)
                .or_insert_with(BTreeSet::new)
                .insert(value);
        }

        ValueMap(value_map)
    }
}

impl CardsCombined<7> {
    pub fn hand_value(&self) -> HandValue {
        self.0
            .into_iter()
            .array_combinations::<5>()
            .collect::<Vec<_>>()
            .par_iter()
            .map(|cards| *cards)
            .map(|cards| CardsCombined(cards))
            .map(From::from)
            .max()
            .expect("At least one combination should exist")
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
        let mut cards = [Card::default(); _];
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
pub type FullBoard = CardsCombined<5>;

impl FullBoard {
    pub fn to_seven(&self, hole: Hole) -> CardsCombined<7> {
        let hole = hole.0;
        let board = self.0;

        CardsCombined([
            hole[0], hole[1], board[0], board[1], board[2], board[3], board[4],
        ])
    }

    pub fn hand_value(&self, hole: Hole) -> HandValue {
        self.to_seven(hole).hand_value()
    }

    pub fn who_wins(&self, h1: Hole, h2: Hole) -> (HandValue, Option<bool>) {
        let (v1, v2) = rayon::join(|| self.hand_value(h1), || self.hand_value(h2));

        match v1.cmp(&v2) {
            Ordering::Greater => (v1, Some(true)),
            Ordering::Less => (v2, Some(false)),
            Ordering::Equal => (v1, None),
        }
    }

    pub fn is_nuts(&self) -> bool {
        match HandValue::from(*self).0 {
            SortedHandValue::RoyalFlush => true,
            SortedHandValue::Quads([Value::Ace, Value::King]) => true,
            SortedHandValue::Quads([_, Value::Ace]) => true,
            SortedHandValue::Straight(Value::Ace) => self
                .0
                .iter()
                .map(Card::suit)
                .counts()
                .values()
                .all(|&c| c <= 2),
            _ => false,
        }
    }
}

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

        if !cards.iter().all_unique() {
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

    pub fn as_full_board(&self) -> Option<FullBoard> {
        if let BoardCards::River { flop, turn, river } = self.0 {
            Some(FullBoard::unchecked([
                flop[0], flop[1], flop[2], turn, river,
            ]))
        } else {
            None
        }
    }

    pub fn is_preflop(&self) -> bool {
        matches!(self.0, BoardCards::Preflop)
    }

    pub fn is_flop(&self) -> bool {
        matches!(self.0, BoardCards::Flop(_))
    }

    pub fn is_turn(&self) -> bool {
        matches!(self.0, BoardCards::Turn { .. })
    }

    pub fn is_river(&self) -> bool {
        matches!(self.0, BoardCards::River { .. })
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct ValueMap(BTreeMap<usize, BTreeSet<Value>>);

impl ValueMap {
    fn to_freq_pairs(&self) -> Vec<(usize, usize)> {
        self.0
            .iter()
            .rev()
            .map(|(&key, values)| (key, values.len()))
            .collect()
    }

    fn to_sorted_values(&self) -> Vec<Value> {
        self.0
            .values()
            .rev()
            .flat_map(|v| v.iter().rev())
            .copied()
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct HandValue(SortedHandValue);

impl PartialOrd for HandValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HandValue {
    fn cmp(&self, other: &Self) -> Ordering {
        use SortedHandValue::*;
        match (self.0, other.0) {
            (RoyalFlush, RoyalFlush) => Ordering::Equal,
            (RoyalFlush, _) => Ordering::Greater,
            (_, RoyalFlush) => Ordering::Less,
            (StraightFlush(v1), StraightFlush(v2)) => v1.cmp(&v2),
            (StraightFlush(_), _) => Ordering::Greater,
            (_, StraightFlush(_)) => Ordering::Less,
            (Quads(v1), Quads(v2)) => v1.cmp(&v2),
            (Quads(_), _) => Ordering::Greater,
            (_, Quads(_)) => Ordering::Less,
            (FullHouse(v1), FullHouse(v2)) => v1.cmp(&v2),
            (FullHouse(_), _) => Ordering::Greater,
            (_, FullHouse(_)) => Ordering::Less,
            (Flush(v1), Flush(v2)) => v1.cmp(&v2),
            (Flush(_), _) => Ordering::Greater,
            (_, Flush(_)) => Ordering::Less,
            (Straight(v1), Straight(v2)) => v1.cmp(&v2),
            (Straight(_), _) => Ordering::Greater,
            (_, Straight(_)) => Ordering::Less,
            (Trips(v1), Trips(v2)) => v1.cmp(&v2),
            (Trips(_), _) => Ordering::Greater,
            (_, Trips(_)) => Ordering::Less,
            (TwoPair(v1), TwoPair(v2)) => v1.cmp(&v2),
            (TwoPair(_), _) => Ordering::Greater,
            (_, TwoPair(_)) => Ordering::Less,
            (OnePair(v1), OnePair(v2)) => v1.cmp(&v2),
            (OnePair(_), _) => Ordering::Greater,
            (_, OnePair(_)) => Ordering::Less,
            (HighCard(v1), HighCard(v2)) => v1.cmp(&v2),
        }
    }
}

impl From<CardsCombined<5>> for HandValue {
    fn from(cards: CardsCombined<5>) -> Self {
        let is_flush = cards.is_flush();
        let is_straight = cards.is_straight();

        if let Some(largest_value) = is_straight {
            if is_flush {
                if largest_value == Value::Ace {
                    Self(SortedHandValue::RoyalFlush)
                } else {
                    Self(SortedHandValue::StraightFlush(largest_value))
                }
            } else {
                Self(SortedHandValue::Straight(largest_value))
            }
        } else if is_flush {
            Self(SortedHandValue::Flush(cards.to_sorted_values()))
        } else {
            let value_map = cards.to_value_map();
            let sorted_values = value_map.to_sorted_values();

            // These unwrapping should not fail with valid poker hands
            match value_map.to_freq_pairs().as_slice() {
                [(4, 1), (1, 1)] => Self(SortedHandValue::Quads(sorted_values.try_into().unwrap())),
                [(3, 1), (2, 1)] => Self(SortedHandValue::FullHouse(
                    sorted_values.try_into().unwrap(),
                )),
                [(3, 1), (1, 2)] => Self(SortedHandValue::Trips(sorted_values.try_into().unwrap())),
                [(2, 2), (1, 1)] => {
                    Self(SortedHandValue::TwoPair(sorted_values.try_into().unwrap()))
                }
                [(2, 1), (1, 3)] => {
                    Self(SortedHandValue::OnePair(sorted_values.try_into().unwrap()))
                }
                [(1, 5)] => Self(SortedHandValue::HighCard(sorted_values.try_into().unwrap())),
                _ => unreachable!(), // Should not happen with valid poker hands
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum SortedHandValue {
    RoyalFlush,
    StraightFlush(Value),
    Quads([Value; 2]),
    FullHouse([Value; 2]),
    Flush([Value; 5]),
    Straight(Value),
    Trips([Value; 3]),
    TwoPair([Value; 3]),
    OnePair([Value; 4]),
    HighCard([Value; 5]),
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
                    DisplayMode::Unicode | DisplayMode::ColoredUnicode => "♥",
                    DisplayMode::ColoredEmoji => "♥️",
                },
                Suit::Diamonds => match self.mode {
                    DisplayMode::Ascii => "d",
                    DisplayMode::Unicode | DisplayMode::ColoredUnicode => "♦",
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
                "{}{}{}{}{}",
                if self.need_ansi() { "\x1b[91m" } else { "" },
                self.card.value(),
                if self.mode.is_unicode() { " " } else { "" },
                self.card.suit().display(self.mode),
                if self.need_ansi() { "\x1b[0m" } else { "" },
            )
        }
    }

    impl CardDisplay {
        fn need_ansi(self) -> bool {
            self.mode == DisplayMode::ColoredUnicode && self.card.is_red()
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
