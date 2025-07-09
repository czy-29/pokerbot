#![allow(dead_code)]

use super::*;
use rand::prelude::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum CashBuyin {
    BB15,
    BB30,
    BB50,
    BB75,
    BB100,
    BB150,
    BB200,
    BB250,
    BB300,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SNGSpeed {
    Fast,
    Turbo,
    Slow,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GameType {
    Cash { buyin: CashBuyin, hands: u16 },
    SNG(SNGSpeed),
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
    // DealHoles(Option<Hole>, Option<Hole>),
    ActionTurn(bool),
    GameOver(GameOver),
    GameAbort,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PlayerEvent {
    Observable(ObservableEvent),
    HeroTurn,
}

impl PlayerEvent {
    fn abort() -> Self {
        Self::Observable(ObservableEvent::GameAbort)
    }
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
    next_button: bool,
    game_over: Option<GameOver>,
    game_abort: bool,
    recv: UnboundedReceiver<PlayerEvent>,
}

impl Player {
    fn new(
        game_type: GameType,
        visibility: Visibility,
        next_button: bool,
        recv: UnboundedReceiver<PlayerEvent>,
    ) -> Self {
        Self {
            game_type,
            visibility,
            next_button,
            game_over: None,
            game_abort: false,
            recv,
        }
    }

    pub fn is_abort(&self) -> bool {
        self.game_abort
    }

    pub fn is_over(&self) -> bool {
        self.game_over.is_some()
    }

    pub fn game_over(&self) -> Option<GameOver> {
        self.game_over
    }

    pub async fn tick_event(&mut self) -> Option<PlayerEvent> {
        if self.is_abort() || self.is_over() {
            return None;
        }

        let event = self.recv.recv().await.unwrap_or(PlayerEvent::abort());
        if let PlayerEvent::Observable(event) = event {
            match event {
                ObservableEvent::GameAbort => {
                    self.game_abort = true;
                }
                ObservableEvent::GameOver(game_over) => {
                    self.game_over = Some(game_over);
                }
                _ => {
                    // todo: restore history
                }
            }
        }

        Some(event)
    }
}

#[derive(Debug)]
pub struct Observer(Player);

impl Observer {
    pub fn is_abort(&self) -> bool {
        self.0.is_abort()
    }

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
    ExitCheckout(u32, u32),
    HandsReached(u32, u32),
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

// todo: make private
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
    pub fn shuffle(&mut self) {
        self.0.shuffle(&mut rand::rng());
    }

    pub fn shuffled(&self) -> Self {
        let mut deck = *self;
        deck.shuffle();
        deck
    }

    pub fn dealer(&self) -> Dealer {
        Dealer(self.0.into_iter())
    }
}

use std::array::IntoIter;

// todo: make private
#[derive(Debug, Clone)]
pub struct Dealer(IntoIter<Card, 52>);

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

#[derive(Debug)]
pub struct Game {
    game_type: GameType,
    game_over: Option<GameOver>,
    player0: PlayerSender,
    player1: PlayerSender,
    observer: Option<PlayerSender>,
    next_button: bool,
}

impl Game {
    pub fn new(game_type: GameType) -> (Self, Player, Player) {
        let p0_vis = Visibility::Player0;
        let p1_vis = Visibility::Player1;
        let (p0_send, p0_recv) = unbounded_channel();
        let (p1_send, p1_recv) = unbounded_channel();
        let next_button = rand::random();
        let game = Self {
            game_type,
            game_over: None,
            player0: PlayerSender {
                visibility: p0_vis,
                send: p0_send,
            },
            player1: PlayerSender {
                visibility: p1_vis,
                send: p1_send,
            },
            observer: None,
            next_button,
        };
        let player0 = Player::new(game_type, p0_vis, next_button, p0_recv);
        let player1 = Player::new(game_type, p1_vis, !next_button, p1_recv);
        (game, player0, player1)
    }

    pub fn observer(&mut self, visibility: Visibility) -> Option<Observer> {
        if self.observer.is_some() {
            return None; // Observer already exists
        }

        let (send, recv) = unbounded_channel();
        let next_button = if visibility == Visibility::Player1 {
            !self.next_button
        } else {
            self.next_button
        };
        self.observer = Some(PlayerSender { visibility, send });
        Some(Observer(Player::new(
            self.game_type,
            visibility,
            next_button,
            recv,
        )))
    }

    pub fn is_over(&self) -> bool {
        self.game_over.is_some()
    }

    pub fn game_over(&self) -> Option<GameOver> {
        self.game_over
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

        if !self.player0.send(event) {
            return Some(true);
        }

        if !self.player1.send(event) {
            return Some(false);
        }

        None
    }

    async fn player_action(&mut self, player0: bool, bounds: ()) -> Result<ObservableEvent, bool> {
        let ob_event = ObservableEvent::ActionTurn(player0);
        self.send_ob(ob_event);
        let send = if player0 {
            &self.player1
        } else {
            &self.player0
        };
        let turn = if player0 {
            &self.player0
        } else {
            &self.player1
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
        self.game_over = Some(game_over);
        let event = ObservableEvent::GameOver(game_over);
        self.send_ob(event);
        self.player0.send(event);
        self.player1.send(event);
        Some(game_over)
    }

    pub async fn run_hand(&mut self) -> Option<GameOver> {
        if self.is_over() {
            return self.game_over();
        }

        let _big_blind = 500;
        let _stack0 = 150000;
        let _stack1 = 150000;
        let _exit_abandon = false;
        let _deck = 0;

        // switch button position
        self.next_button = !self.next_button;

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
