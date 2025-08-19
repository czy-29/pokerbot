use indexmap::IndexSet;
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
    const ACE_HIGH: u8 = 13;

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

    fn as_u8_straight(self) -> u8 {
        self.as_u8() + 1
    }

    fn from_u8_straight(value: u8) -> Self {
        match value {
            0 | 13 => Self::Ace,
            1 => Self::Deuce,
            2 => Self::Trey,
            3 => Self::Four,
            4 => Self::Five,
            5 => Self::Six,
            6 => Self::Seven,
            7 => Self::Eight,
            8 => Self::Nine,
            9 => Self::Ten,
            10 => Self::Jack,
            11 => Self::Queen,
            12 => Self::King,
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

    pub fn contains_value(&self, value: Value) -> bool {
        self.0.iter().map(Card::value).contains(&value)
    }

    pub fn contains_suit(&self, suit: Suit) -> bool {
        self.0.iter().map(Card::suit).contains(&suit)
    }

    pub fn contains_card(&self, card: Card) -> bool {
        self.0.contains(&card)
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
            Some(Value::from_u8_straight(u8s[N - 1]))
        } else {
            None
        }
    }

    fn is_straight(&self) -> Option<Value> {
        let mut u8s = self.0.map(|card| card.value().as_u8_straight());
        let check_straight = Self::check_straight(u8s);

        if check_straight.is_none() && u8s.contains(&Value::ACE_HIGH) {
            // Check for wheel (A-2-3-4-5)
            for u in &mut u8s {
                if *u == Value::ACE_HIGH {
                    *u = 0;
                    break;
                }
            }

            return Self::check_straight(u8s);
        }

        check_straight
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

impl Hole {
    pub fn is_pocket_pair(&self) -> bool {
        self.0.iter().map(Card::value).all_equal()
    }

    pub fn is_pocket(&self, value: Value) -> bool {
        self.0.iter().map(Card::value).all(|v| v == value)
    }

    pub fn is_suited(&self) -> bool {
        self.is_flush()
    }

    fn is_of_values(&self, values: [Value; 2]) -> bool {
        self.contains_value(values[0]) && self.contains_value(values[1])
    }

    fn from_values_suited(values: [Value; 2], suit: Suit) -> Self {
        Self([Card(values[0], suit), Card(values[1], suit)])
    }
}

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

    pub fn is_nuts(&self, hole: Hole) -> bool {
        self.find_nuts() == hole
    }

    pub fn find_nuts(&self) -> FindNuts {
        let cards = self.to_vec();
        let board_paired = Self::paired(&cards);

        if let Some((suit, cards)) = Self::flush_cards(&cards) {
            let cards_len = cards.len();
            let (nuts_high_value, sf_solves) = Self::straight_scan(&cards, false);
            let nuts_high_card = Card(nuts_high_value, suit);
            let mut sf_solves = sf_solves.into_iter();

            match sf_solves.next() {
                None => {
                    if board_paired {
                        Self::quads_full_house(&cards)
                    } else if cards_len == 3 {
                        FindNuts::CardPlusAnySuited(nuts_high_card)
                    } else {
                        FindNuts::CardPlusAny(nuts_high_card)
                    }
                }
                Some(StraightSolve::None) => FindNuts::AnyTwo,
                Some(StraightSolve::One(value)) => FindNuts::CardPlusAny(Card(value, suit)),
                Some(StraightSolve::Two(sf0)) => {
                    let sf0_hole = Hole::from_values_suited(sf0, suit);

                    match sf_solves.next() {
                        None => {
                            if board_paired {
                                FindNuts::OneHole(sf0_hole)
                            } else if sf0[0] == nuts_high_value {
                                if cards_len == 3 {
                                    FindNuts::CardPlusAnySuited(nuts_high_card)
                                } else {
                                    FindNuts::CardPlusAny(nuts_high_card)
                                }
                            } else {
                                FindNuts::ThreeHoles([
                                    sf0_hole,
                                    Hole::unchecked([nuts_high_card, sf0_hole[0]]),
                                    Hole::unchecked([nuts_high_card, sf0_hole[1]]),
                                ])
                            }
                        }
                        Some(StraightSolve::None) => unreachable!(), // Should not happen
                        Some(StraightSolve::One(value)) => FindNuts::CardPlusAny(Card(value, suit)),
                        Some(StraightSolve::Two(sf1)) => {
                            let sf1_hole = Hole::from_values_suited(sf1, suit);
                            let ace = Card(Value::Ace, suit);

                            if sf1[0] != sf0[1] {
                                if board_paired || sf0[0] != nuts_high_value {
                                    FindNuts::OneHole(sf0_hole)
                                } else {
                                    FindNuts::ThreeHoles([
                                        sf0_hole,
                                        Hole::unchecked([nuts_high_card, sf1_hole[0]]),
                                        Hole::unchecked([nuts_high_card, sf1_hole[1]]),
                                    ])
                                }
                            } else if board_paired {
                                FindNuts::TwoHoles([sf0_hole, sf1_hole])
                            } else if sf0[0] == nuts_high_value {
                                FindNuts::ThreeHoles([
                                    sf0_hole,
                                    sf1_hole,
                                    Hole::unchecked([nuts_high_card, sf1_hole[1]]),
                                ])
                            } else if let Some(sf2) = sf_solves.next() {
                                if let Some(last) = sf2.last()
                                    && last == Value::Ace
                                {
                                    FindNuts::ThreeHoles([
                                        sf0_hole,
                                        sf1_hole,
                                        Hole::unchecked([ace, sf1_hole[0]]),
                                    ])
                                } else {
                                    FindNuts::TwoHoles([sf0_hole, sf1_hole])
                                }
                            } else if sf1[1] == Value::Ace {
                                FindNuts::ThreeHoles([
                                    sf0_hole,
                                    sf1_hole,
                                    Hole::unchecked([ace, sf0_hole[0]]),
                                ])
                            } else {
                                FindNuts::ThreeHoles([
                                    sf0_hole,
                                    sf1_hole,
                                    Hole::unchecked([nuts_high_card, sf1_hole[0]]),
                                ])
                            }
                        }
                    }
                }
            }
        } else if board_paired {
            Self::quads_full_house(&cards)
        } else {
            let (_, straight) = Self::straight_scan(&cards, true);

            match straight.first() {
                Some(StraightSolve::None) => FindNuts::AnyTwo,
                Some(StraightSolve::One(value)) => FindNuts::OneValue(*value),
                Some(StraightSolve::Two(values)) => FindNuts::TwoValues(*values),
                None => {
                    FindNuts::PocketPair(cards.iter().map(Card::value).max().unwrap_or(Value::Ace))
                }
            }
        }
    }

    fn flush_cards(cards: &[Card]) -> Option<(Suit, Vec<Card>)> {
        cards
            .iter()
            .copied()
            .into_group_map_by(Card::suit)
            .into_iter()
            .find(|(_, cards)| cards.len() >= 3)
    }

    fn straight_scan(cards: &[Card], only_first: bool) -> (Value, IndexSet<StraightSolve>) {
        let mut values: BTreeSet<u8> = cards
            .iter()
            .map(Card::value)
            .map(Value::as_u8_straight)
            .collect();

        if values.contains(&Value::ACE_HIGH) {
            values.insert(0); // For wheel (A-2-3-4-5)
        }

        let range_start = *values
            .first()
            .expect("Input cards should not be empty")
            .max(&2)
            - 2;
        let range_end =
            Value::ACE_HIGH.min(values.last().expect("Input cards should not be empty") + 2) - 4;
        let remain_set: BTreeSet<u8> = (0..=13)
            .collect::<BTreeSet<u8>>()
            .difference(&values)
            .copied()
            .collect();
        let remain_high = remain_set
            .last()
            .copied()
            .map(Value::from_u8_straight)
            .expect("Input cards should at most contain 12 cards");
        let mut solves = IndexSet::new();

        for start in (range_start..=range_end).rev() {
            let mut solve: Vec<u8> = Vec::with_capacity(5);
            solve.extend(remain_set.intersection(&(start..start + 5).collect()));

            if solve.len() <= 2 {
                match solve.len() {
                    0 => {
                        solves.insert(StraightSolve::None);
                        return (remain_high, solves);
                    }
                    1 => {
                        solves.insert(StraightSolve::One(Value::from_u8_straight(solve[0])));
                    }
                    2 => {
                        let low = Value::from_u8_straight(solve[0]);
                        let high = Value::from_u8_straight(solve[1]);

                        solves.insert(StraightSolve::Two([high, low]));
                    }
                    _ => unreachable!(), // Should not happen with valid poker hands
                }

                if only_first {
                    return (remain_high, solves);
                }
            }
        }

        (remain_high, solves)
    }

    fn paired(cards: &[Card]) -> bool {
        !cards.iter().map(Card::value).all_unique()
    }

    fn quads_full_house(cards: &[Card]) -> FindNuts {
        let value_map: ValueMap = cards.into();
        let sorted_values = value_map.to_sorted_values();

        match value_map.to_count_pairs().as_slice() {
            [(3, 1)] | [(3, 1), (1, _)] => FindNuts::OneValue(sorted_values[0]),
            [(2, 1), (1, _)] => {
                let pair = sorted_values[0];
                let single_high = sorted_values[1];

                if pair > single_high {
                    FindNuts::PocketOrTwo(pair, [pair, single_high])
                } else {
                    FindNuts::PocketPair(pair)
                }
            }
            [(4, 1)] => {
                if sorted_values[0] == Value::Ace {
                    FindNuts::OneValue(Value::King)
                } else {
                    FindNuts::OneValue(Value::Ace)
                }
            }
            [(2, 2)] => {
                let high = sorted_values[0];
                let low = sorted_values[1];

                FindNuts::PocketOrTwo(high, [high, low])
            }
            [(4, 1), (1, 1)] => {
                let quad = sorted_values[0];
                let kicker = sorted_values[1];

                if quad == Value::Ace {
                    if kicker == Value::King {
                        FindNuts::AnyTwo
                    } else {
                        FindNuts::OneValue(Value::King)
                    }
                } else {
                    if kicker == Value::Ace {
                        FindNuts::AnyTwo
                    } else {
                        FindNuts::OneValue(Value::Ace)
                    }
                }
            }
            [(3, 1), (2, 1)] => {
                let trip = sorted_values[0];
                let pair = sorted_values[1];

                if pair > trip {
                    FindNuts::PocketOrTwo(pair, [pair, trip])
                } else {
                    FindNuts::OneValue(trip)
                }
            }
            [(2, 2), (1, 1)] => {
                let pair_high = sorted_values[0];
                let pair_low = sorted_values[1];
                let single = sorted_values[2];

                if pair_low > single {
                    FindNuts::PocketOrTwo(pair_high, [pair_high, pair_low])
                } else {
                    FindNuts::PocketPair(pair_high)
                }
            }
            _ => unreachable!(),
        }
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum StraightSolve {
    None,
    One(Value),
    Two([Value; 2]),
}

impl StraightSolve {
    fn last(&self) -> Option<Value> {
        match self {
            Self::None => None,
            Self::One(value) => Some(*value),
            Self::Two(values) => Some(values[1]),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum FindNuts {
    PocketPair(Value),
    OneValue(Value),
    TwoValues([Value; 2]),
    PocketOrTwo(Value, [Value; 2]),
    OneHole(Hole),
    TwoHoles([Hole; 2]),
    ThreeHoles([Hole; 3]),
    CardPlusAny(Card),
    CardPlusAnySuited(Card),
    AnyTwo,
}

impl Default for FindNuts {
    fn default() -> Self {
        Self::PocketPair(Value::Ace)
    }
}

impl PartialEq<Hole> for FindNuts {
    fn eq(&self, other: &Hole) -> bool {
        match *self {
            Self::PocketPair(v) => other.is_pocket(v),
            Self::OneValue(v) => other.contains_value(v),
            Self::TwoValues(v) => other.is_of_values(v),
            Self::PocketOrTwo(v, v2) => other.is_pocket(v) || other.is_of_values(v2),
            Self::OneHole(hole) => hole == *other,
            Self::TwoHoles(holes) => holes.contains(other),
            Self::ThreeHoles(holes) => holes.contains(other),
            Self::CardPlusAny(card) => other.contains_card(card),
            Self::CardPlusAnySuited(card) => other.contains_card(card) && other.is_suited(),
            Self::AnyTwo => true,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct ValueMap(BTreeMap<usize, BTreeSet<Value>>);

impl From<&[Card]> for ValueMap {
    fn from(cards: &[Card]) -> Self {
        let mut value_map: BTreeMap<usize, BTreeSet<Value>> = BTreeMap::new();

        for (value, count) in cards.iter().map(Card::value).counts() {
            value_map
                .entry(count)
                .or_insert_with(BTreeSet::new)
                .insert(value);
        }

        Self(value_map)
    }
}

impl ValueMap {
    fn to_count_pairs(&self) -> Vec<(usize, usize)> {
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

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct HandValue(SortedHandValue);

impl Deref for HandValue {
    type Target = SortedHandValue;

    fn deref(&self) -> &Self::Target {
        &self.0
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
            let value_map: ValueMap = cards.as_slice().into();
            let sorted_values = value_map.to_sorted_values();

            // These unwrapping should not fail with valid poker hands
            match value_map.to_count_pairs().as_slice() {
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

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SortedHandValue {
    #[default]
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

impl PartialOrd for SortedHandValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SortedHandValue {
    fn cmp(&self, other: &Self) -> Ordering {
        use SortedHandValue::*;
        match (self, other) {
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
