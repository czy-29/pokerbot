#![allow(dead_code)]

use super::*;
use rand::prelude::*;
use std::{array, vec};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum CashBuyin {
    BB15,
    BB30,
    BB50,
    BB75,
    #[default]
    BB100,
    BB150,
    BB200,
    BB250,
    BB300,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SNGSpeed {
    Turbo,
    Medium,
    #[default]
    Slow,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GameType {
    Cash { buyin: CashBuyin, hands: u16 },
    SNG(SNGSpeed),
}

impl Default for GameType {
    fn default() -> Self {
        Self::SNG(Default::default())
    }
}

impl GameType {
    pub fn cash_default() -> Self {
        Self::Cash {
            buyin: CashBuyin::default(),
            hands: 0,
        }
    }

    fn is_sng(self) -> bool {
        matches!(self, Self::SNG(_))
    }

    fn hands_limit(self) -> Option<u16> {
        match self {
            Self::Cash { hands, .. } => {
                if hands == 0 {
                    None
                } else {
                    Some(hands)
                }
            }
            Self::SNG(_) => None,
        }
    }

    fn init_stack(self) -> u32 {
        match self {
            Self::Cash { buyin, .. } => match buyin {
                CashBuyin::BB15 => 7500,
                CashBuyin::BB30 => 15000,
                CashBuyin::BB50 => 25000,
                CashBuyin::BB75 => 37500,
                CashBuyin::BB100 => 50000,
                CashBuyin::BB150 => 75000,
                CashBuyin::BB200 => 100000,
                CashBuyin::BB250 => 125000,
                CashBuyin::BB300 => 150000,
            },
            Self::SNG(speed) => match speed {
                SNGSpeed::Turbo => 3000,
                SNGSpeed::Medium => 7500,
                SNGSpeed::Slow => 15000,
            },
        }
    }

    fn blind_levels(self) -> vec::IntoIter<u16> {
        match self {
            Self::Cash { .. } => vec![500],
            Self::SNG(speed) => match speed {
                SNGSpeed::Turbo => vec![50, 100, 150, 200],
                SNGSpeed::Medium => vec![50, 100, 150, 200, 300, 400, 500],
                SNGSpeed::Slow => vec![50, 100, 150, 200, 300, 400, 500, 600, 800, 1000],
            },
        }
        .into_iter()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Visibility {
    None,
    Player0,
    Player1,
    God,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ObservableEvent {
    DealHoles([Option<Hole>; 2]),
    ActionTurn(bool),
    GameOver(GameOver),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PlayerEvent {
    Observable(ObservableEvent),
    HeroTurn,
}

impl PlayerEvent {
    const fn unwrap_observable(self) -> ObservableEvent {
        match self {
            Self::Observable(observable) => observable,
            Self::HeroTurn => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Player {
    game_type: GameType,
    visibility: Visibility,
    recv: UnboundedReceiver<PlayerEvent>,
    heads_up: HeadsUp,
}

impl Player {
    fn new(
        game_type: GameType,
        visibility: Visibility,
        recv: UnboundedReceiver<PlayerEvent>,
        button: bool,
    ) -> Self {
        Self {
            game_type,
            visibility,
            recv,
            heads_up: HeadsUp::new(game_type, button),
        }
    }

    pub fn is_over(&self) -> bool {
        self.heads_up.is_over()
    }

    pub fn game_over(&self) -> Option<GameOver> {
        self.heads_up.game_over()
    }

    pub async fn tick_event(&mut self) -> Option<PlayerEvent> {
        if self.is_over() {
            return None;
        }

        let event = self.recv.recv().await.unwrap_or(self.heads_up.abort());
        if let PlayerEvent::Observable(event) = event {
            self.heads_up.event(event);
        }

        Some(event)
    }
}

#[derive(Debug)]
pub struct Observer(Player);

impl Observer {
    pub fn is_over(&self) -> bool {
        self.0.is_over()
    }

    pub fn game_over(&self) -> Option<GameOver> {
        self.0.game_over()
    }

    pub async fn tick_event(&mut self) -> Option<ObservableEvent> {
        self.0
            .tick_event()
            .await
            .map(PlayerEvent::unwrap_observable)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GameOver {
    Defeated(bool),
    ExitAbandon(bool),
    ExitCheckout(bool, [u32; 2]),
    AbortCheckout([u32; 2]),
    HandsReached([u32; 2]),
    GameAbort,
}

#[derive(Debug)]
struct PlayerSender {
    visibility: Visibility,
    send: UnboundedSender<PlayerEvent>,
}

impl PlayerSender {
    fn send(&self, event: ObservableEvent) -> bool {
        // todo: transform event (God |-> FirstPerson)
        self.send.send(PlayerEvent::Observable(event)).is_ok()
    }

    async fn turn(&self, _bounds: ()) -> Option<ObservableEvent> {
        todo!()
    }
}

// todo: make private, inside run_hand
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

impl Deck {
    pub fn shuffle_and_deal(&mut self) -> Dealer {
        self.0.shuffle(&mut rand::rng());
        Dealer(self.0.into_iter())
    }
}

// todo: make private, inside run_hand
#[derive(Debug, Clone)]
pub struct Dealer(array::IntoIter<Card, 52>);

impl Dealer {
    pub fn deal_card(&mut self) -> Card {
        self.0.next().unwrap()
    }

    pub fn deal_hole(&mut self) -> Hole {
        Hole::unchecked([self.deal_card(), self.deal_card()])
    }

    pub fn deal_flop(&mut self) -> Flop {
        Flop::unchecked([self.deal_card(), self.deal_card(), self.deal_card()])
    }
}

// todo: HeadsUp: core gameplay, rules, logic, and state machine.
#[derive(Debug, Clone)]
struct HeadsUp {
    game_over: Option<GameOver>,

    // game info
    is_sng: bool,
    hands_limit: Option<u16>,
    blind_levels: vec::IntoIter<u16>,

    // current state
    cur_blind: u16,
    button: bool,
    stacks: [u32; 2],
    pot: u32,
    cur_round: [u32; 2],
    behinds: [u32; 2],
    events: Vec<ObservableEvent>,
}

impl HeadsUp {
    fn new(game_type: GameType, button: bool) -> Self {
        let init_stack = game_type.init_stack();
        let stacks = [init_stack, init_stack];
        let mut blind_levels = game_type.blind_levels();
        let cur_blind = blind_levels.next().unwrap(); // always has one

        Self {
            game_over: None,
            is_sng: game_type.is_sng(),
            hands_limit: game_type.hands_limit(),
            blind_levels,
            cur_blind,
            button,
            stacks,
            pot: 0,
            cur_round: [0, 0],
            behinds: stacks,
            events: Default::default(),
        }
    }

    fn is_over(&self) -> bool {
        self.game_over.is_some()
    }

    fn game_over(&self) -> Option<GameOver> {
        self.game_over
    }

    fn abort(&self) -> PlayerEvent {
        PlayerEvent::Observable(ObservableEvent::GameOver(if self.is_sng {
            GameOver::GameAbort
        } else {
            GameOver::AbortCheckout(self.stacks)
        }))
    }

    fn set_game_over(&mut self, game_over: GameOver) {
        self.game_over = Some(game_over);
    }

    fn deal_holes(&mut self, holes: [Hole; 2]) -> ObservableEvent {
        // todo: blinds betting, save holes, other possibilities
        ObservableEvent::DealHoles([Some(holes[0]), Some(holes[1])])
    }

    fn event(&mut self, event: ObservableEvent) {
        self.events.push(event);
        match event {
            ObservableEvent::GameOver(game_over) => {
                self.set_game_over(game_over);
            }
            ObservableEvent::DealHoles(_holes) => {
                // todo: blinds betting, save holes, other possibilities
            }
            _ => {
                // todo: restore history
            }
        }
    }
}

#[derive(Debug)]
pub struct Game {
    game_type: GameType,
    init_button: bool,
    players: [PlayerSender; 2],
    observer: Option<PlayerSender>,
    deck: Deck,
    heads_up: HeadsUp,
}

impl Game {
    pub fn new(game_type: GameType) -> (Self, [Player; 2]) {
        let vis = [Visibility::Player0, Visibility::Player1];
        let [(send0, recv0), (send1, recv1)] = [unbounded_channel(), unbounded_channel()];
        let init_button = rand::random();
        let game = Self {
            game_type,
            init_button,
            players: [
                PlayerSender {
                    visibility: vis[0],
                    send: send0,
                },
                PlayerSender {
                    visibility: vis[1],
                    send: send1,
                },
            ],
            observer: None,
            deck: Default::default(),
            heads_up: HeadsUp::new(game_type, init_button),
        };
        let players = [
            Player::new(game_type, vis[0], recv0, init_button),
            Player::new(game_type, vis[1], recv1, !init_button),
        ];
        (game, players)
    }

    pub fn observer(&mut self, visibility: Visibility) -> Option<Observer> {
        if self.observer.is_some() {
            return None; // Observer already exists
        }

        let (send, recv) = unbounded_channel();
        let button = if visibility == Visibility::Player1 {
            !self.init_button
        } else {
            self.init_button
        };
        self.observer = Some(PlayerSender { visibility, send });
        Some(Observer(Player::new(
            self.game_type,
            visibility,
            recv,
            button,
        )))
    }

    pub fn is_over(&self) -> bool {
        self.heads_up.is_over()
    }

    pub fn game_over(&self) -> Option<GameOver> {
        self.heads_up.game_over()
    }

    fn send_ob(&mut self, event: ObservableEvent) {
        if let Some(observer) = &self.observer {
            if !observer.send(event) {
                self.observer = None;
            }
        }
    }

    fn dispatch_event(&mut self, event: ObservableEvent) -> Option<bool> {
        self.send_ob(event);

        if !self.players[0].send(event) {
            return Some(true);
        }

        if !self.players[1].send(event) {
            return Some(false);
        }

        None
    }

    async fn player_action(&mut self, player0: bool, bounds: ()) -> Result<ObservableEvent, bool> {
        let ob_event = ObservableEvent::ActionTurn(player0);
        self.send_ob(ob_event);
        let send = if player0 {
            &self.players[1]
        } else {
            &self.players[0]
        };
        let turn = if player0 {
            &self.players[0]
        } else {
            &self.players[1]
        };

        // Err(true) for player0 crashing,
        // Err(false) for player1 crashing

        if !send.send(ob_event) {
            return Err(!player0);
        }

        turn.turn(bounds).await.ok_or(player0)
    }

    // infallible game over
    fn send_game_over(&mut self, game_over: GameOver) -> Option<GameOver> {
        self.heads_up.set_game_over(game_over);
        let event = ObservableEvent::GameOver(game_over);
        self.send_ob(event);
        self.players[0].send(event);
        self.players[1].send(event);
        Some(game_over)
    }

    pub async fn run_hand(&mut self) -> Option<GameOver> {
        if self.is_over() {
            return self.game_over();
        }

        let mut dealer = self.deck.shuffle_and_deal();

        // todo: result handling for the 2 statements
        let deal_holes = self
            .heads_up
            .deal_holes([dealer.deal_hole(), dealer.deal_hole()]);
        self.dispatch_event(deal_holes);

        // let button = self.next_button;
        let _big_blind = 500;
        let _stack0 = 150000;
        let _stack1 = 150000;
        let _exit_abandon = false;
        let _deck = 0;

        // switch button position
        // self.next_button = !button;

        None
    }

    pub async fn run(mut self) -> GameOver {
        loop {
            if let Some(game_over) = self.run_hand().await {
                return game_over;
            }
        }
    }
}
