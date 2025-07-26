#![allow(dead_code)]

use super::*;
use rand::prelude::*;
use std::{array, ops::RangeInclusive, vec};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    oneshot::{Sender, channel},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Action(ActionValue);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ActionValue {
    Exit,
    Fold,
    CheckOrCall,
    BetOrRaise(u32),
    AllIn,
}

impl Action {
    pub fn exit() -> Self {
        Self(ActionValue::Exit)
    }

    pub fn fold() -> Self {
        Self(ActionValue::Fold)
    }

    pub fn check_or_call() -> Self {
        Self(ActionValue::CheckOrCall)
    }

    pub fn bet_or_raise(amount: u32) -> Option<Self> {
        if amount == 0 || amount % 25 != 0 {
            None // Invalid bet or raise amount
        } else {
            Some(Self(ActionValue::BetOrRaise(amount)))
        }
    }

    pub fn all_in() -> Self {
        Self(ActionValue::AllIn)
    }

    pub fn value(&self) -> ActionValue {
        self.0
    }

    fn is_exit(&self) -> bool {
        matches!(self.0, ActionValue::Exit)
    }

    fn is_fold(&self) -> bool {
        matches!(self.0, ActionValue::Fold)
    }

    fn is_check_or_call(&self) -> bool {
        matches!(self.0, ActionValue::CheckOrCall)
    }

    fn is_all_in(&self) -> bool {
        matches!(self.0, ActionValue::AllIn)
    }
}

impl FromStr for Action {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "e" | "x" => Ok(Self::exit()),
            "f" => Ok(Self::fold()),
            "c" => Ok(Self::check_or_call()),
            "a" => Ok(Self::all_in()),
            s => s
                .parse::<u32>()
                .map_err(|_| ())
                .and_then(|amount| Self::bet_or_raise(amount).ok_or(())),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ActionSendError {
    NotHeroTurn,
    InvalidAction,
    GameAbort(GameOver),
}

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
    Player(bool), // true for player 0, false for player 1
    God,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ObservableEvent {
    DealHoles([Option<Hole>; 2]),
    ShowdownAll([Hole; 2]),
    ShowdownAuto([Hole; 2]), // board nuts auto chop
    PlayerAction(Action),
    GameOver(GameOver),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum PlayerEvent {
    Observable(ObservableEvent),
    HeroTurn(BetBound),
}

impl PlayerEvent {
    const fn unwrap_observable(self) -> ObservableEvent {
        match self {
            Self::Observable(observable) => observable,
            Self::HeroTurn(_) => unreachable!(),
        }
    }
}

#[derive(Debug)]
enum InternalEvent {
    Observable(ObservableEvent),
    HeroTurn(BetBound, Sender<Action>),
}

impl InternalEvent {
    fn take_player(self) -> (PlayerEvent, Option<(BetBound, Sender<Action>)>) {
        match self {
            Self::Observable(event) => (PlayerEvent::Observable(event), None),
            Self::HeroTurn(bet_bound, sender) => (
                PlayerEvent::HeroTurn(bet_bound.clone()),
                Some((bet_bound, sender)),
            ),
        }
    }
}

#[derive(Debug)]
pub struct Player {
    game_type: GameType,
    visibility: Visibility,
    recv: UnboundedReceiver<InternalEvent>,
    hero_turn: Option<(BetBound, Sender<Action>)>,
    heads_up: HeadsUp,
}

impl Player {
    fn new(
        game_type: GameType,
        visibility: Visibility,
        recv: UnboundedReceiver<InternalEvent>,
        button: bool,
    ) -> Self {
        Self {
            game_type,
            visibility,
            recv,
            hero_turn: None,
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

        let (event, hero_turn) = self
            .recv
            .recv()
            .await
            .unwrap_or(InternalEvent::Observable(ObservableEvent::GameOver(
                self.heads_up.abort(),
            )))
            .take_player();

        self.hero_turn = hero_turn;
        if let PlayerEvent::Observable(event) = event {
            self.heads_up.event(event);
        }

        Some(event)
    }

    pub fn send_action(&mut self, action: Action) -> Result<(), ActionSendError> {
        if self.hero_turn.is_none() {
            return Err(ActionSendError::NotHeroTurn);
        }

        // hero_turn is guaranteed to be Some here
        let Some(action) = self.hero_turn.as_ref().unwrap().0.alter_eq(action) else {
            return Err(ActionSendError::InvalidAction);
        };

        // hero_turn is guaranteed to be Some here
        if self.hero_turn.take().unwrap().1.send(action).is_err() {
            let game_over = self.heads_up.abort();
            self.heads_up.event(ObservableEvent::GameOver(game_over));
            return Err(ActionSendError::GameAbort(game_over));
        }

        Ok(())
    }

    pub fn parse_send_action(&mut self, action: &str) -> Result<(), ActionSendError> {
        self.send_action(action.parse().map_err(|_| ActionSendError::InvalidAction)?)
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
    send: UnboundedSender<InternalEvent>,
}

impl PlayerSender {
    fn send(&self, event: ObservableEvent) -> bool {
        // todo: transform event (God |-> FirstPerson)
        self.send.send(InternalEvent::Observable(event)).is_ok()
    }

    async fn turn(&self, bet_bound: BetBound) -> Option<Action> {
        let (send, recv) = channel();

        if self
            .send
            .send(InternalEvent::HeroTurn(bet_bound, send))
            .is_err()
        {
            return None; // Player crashed
        }

        recv.await.ok()
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
        // Always has cards left, guaranteed by the game logic
        self.0.next().unwrap()
    }

    pub fn deal_hole(&mut self) -> Hole {
        Hole::unchecked([self.deal_card(), self.deal_card()])
    }

    pub fn deal_flop(&mut self) -> Flop {
        Flop::unchecked([self.deal_card(), self.deal_card(), self.deal_card()])
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum BetBound {
    FoldCheckAllin,
    FoldCheckBetAllin(RangeInclusive<u32>),
    FoldAllin,
    FoldCall,
    FoldCallAllin,
    FoldCallRaiseAllin(RangeInclusive<u32>),
    FoldBetAllin(RangeInclusive<u32>), // river nuts button(!opened)
    FoldRaiseAllin(RangeInclusive<u32>), // river nuts opened
}

impl BetBound {
    pub fn validate_action(&self, action: Action) -> bool {
        if action.is_exit() || action.is_fold() {
            return true; // always valid
        }

        match self {
            Self::FoldCheckAllin | Self::FoldCallAllin => {
                action.is_check_or_call() || action.is_all_in()
            }
            Self::FoldCheckBetAllin(range) | Self::FoldCallRaiseAllin(range) => {
                if let ActionValue::BetOrRaise(amount) = action.value() {
                    range.contains(&amount)
                } else {
                    action.is_check_or_call() || action.is_all_in()
                }
            }
            Self::FoldAllin => action.is_all_in(),
            Self::FoldCall => action.is_check_or_call(),
            Self::FoldBetAllin(range) | Self::FoldRaiseAllin(range) => {
                if let ActionValue::BetOrRaise(amount) = action.value() {
                    range.contains(&amount)
                } else {
                    action.is_all_in()
                }
            }
        }
    }

    pub fn alter_eq(&self, action: Action) -> Option<Action> {
        if !self.validate_action(action) {
            return None; // Invalid action
        }

        if let ActionValue::BetOrRaise(amount) = action.value() {
            match self {
                Self::FoldCheckBetAllin(range)
                | Self::FoldCallRaiseAllin(range)
                | Self::FoldBetAllin(range)
                | Self::FoldRaiseAllin(range) => {
                    if amount == *range.end() {
                        return Some(Action::all_in());
                    }
                }
                _ => unreachable!(),
            }
        }

        Some(action)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum ActionOver {
    TurnOver,
    RoundOver,
    ShowdownAll,
    ShowndownRiver,
    HandOver,
    GameOver(GameOver),
}

// todo: HeadsUp: core gameplay, rules, logic, and state machine.
#[derive(Debug, Clone)]
struct HeadsUp {
    game_over: Option<GameOver>,

    // game info
    is_sng: bool,
    hands_limit: Option<u16>,
    blind_levels: vec::IntoIter<u16>,

    // current hand state
    cur_blind: u16,
    cur_turn: bool,
    button: bool,
    stacks: [u32; 2],
    pot: u32,
    cur_round: [u32; 2],
    behinds: [u32; 2],
    last_bet: u32,
    last_aggressor: bool,
    opened: bool,
    holes: [Option<Hole>; 2],
    board: Board,
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
            cur_turn: button,
            button,
            stacks,
            pot: 0,
            cur_round: [0, 0],
            behinds: stacks,
            last_bet: 0,
            last_aggressor: button,
            opened: false,
            holes: [None, None],
            board: Default::default(),
            events: Default::default(),
        }
    }

    fn is_over(&self) -> bool {
        self.game_over.is_some()
    }

    fn game_over(&self) -> Option<GameOver> {
        self.game_over
    }

    fn abort(&self) -> GameOver {
        if self.is_sng {
            GameOver::GameAbort
        } else {
            GameOver::AbortCheckout(self.stacks)
        }
    }

    fn force_exit(&self, player: bool) -> GameOver {
        if self.is_sng {
            GameOver::ExitAbandon(player)
        } else {
            GameOver::ExitCheckout(player, self.stacks)
        }
    }

    fn set_game_over(&mut self, game_over: GameOver) {
        self.game_over = Some(game_over);
    }

    fn set_holes(&mut self, holes: [Hole; 2]) {
        self.holes = [Some(holes[0]), Some(holes[1])];
    }

    fn big_blind(&self) -> u32 {
        self.cur_blind as u32
    }

    // todo: river nuts
    fn bet_bound(&self) -> BetBound {
        let hero = if self.cur_turn { 0 } else { 1 };
        let behind = self.behinds[hero];

        // can check
        if self.cur_round[0] == 0 && self.cur_round[1] == 0 {
            let big_blind = self.big_blind();

            return if behind <= big_blind {
                BetBound::FoldCheckAllin
            } else {
                BetBound::FoldCheckBetAllin(big_blind..=behind)
            };
        }

        let villain = 1 - hero;
        let villian_bet = self.cur_round[villain];

        // cover
        if behind <= villian_bet {
            return BetBound::FoldAllin;
        }

        // villian all in
        if self.behinds[villain] == villian_bet {
            return BetBound::FoldCall;
        }

        let min_raise = villian_bet + (villian_bet - self.last_bet);

        // call or all in
        if behind <= min_raise {
            return BetBound::FoldCallAllin;
        }

        BetBound::FoldCallRaiseAllin(min_raise..=behind)
    }

    fn effective_behind(&self) -> u32 {
        self.behinds[0].min(self.behinds[1])
    }

    fn deal_holes(&mut self, holes: [Hole; 2]) -> Option<(bool, BetBound)> {
        self.set_holes(holes);
        self.deal_holes_int()?;
        Some((self.cur_turn, self.bet_bound()))
    }

    fn deal_holes_int(&mut self) -> Option<()> {
        let effective_stack = self.effective_behind();
        let big_blind = self.big_blind();
        let small_blind = big_blind / 2;

        // forced all in
        if effective_stack <= small_blind {
            self.pot += effective_stack * 2;
            self.behinds[0] -= effective_stack;
            self.behinds[1] -= effective_stack;
            return None;
        }

        let sb = if self.button { 0 } else { 1 };
        let bb = 1 - sb;

        // blinds betting
        self.cur_round[sb] = small_blind;
        self.cur_round[bb] = big_blind.min(self.behinds[bb]);

        Some(())
    }

    fn action(&mut self, _action: Action) -> ActionOver {
        todo!() // Implement action logic
    }

    fn event(&mut self, event: ObservableEvent) {
        self.events.push(event);
        match event {
            ObservableEvent::GameOver(game_over) => {
                self.set_game_over(game_over);
            }
            ObservableEvent::DealHoles(holes) => {
                self.holes = holes;
                self.deal_holes_int();
            }
            ObservableEvent::ShowdownAll(holes) => {
                self.set_holes(holes);
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
        let vis = [Visibility::Player(true), Visibility::Player(false)];
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
        let button = if visibility == Visibility::Player(false) {
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

    // None for crashing
    async fn player_action(&mut self, cur_turn: bool, bet_bound: BetBound) -> Option<Action> {
        self.players[if cur_turn { 0 } else { 1 }]
            .turn(bet_bound)
            .await
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

    async fn run_bet_round(&mut self) {
        todo!() // Implement betting round logic
    }

    pub async fn run_hand(&mut self) -> Option<GameOver> {
        if self.is_over() {
            return self.game_over();
        }

        let mut dealer = self.deck.shuffle_and_deal();

        let holes = [dealer.deal_hole(), dealer.deal_hole()];
        let bet_info = self.heads_up.deal_holes(holes);
        let mut _showdown_all = bet_info.is_none();

        if let Some(player) =
            self.dispatch_event(ObservableEvent::DealHoles([Some(holes[0]), Some(holes[1])]))
        {
            return self.send_game_over(self.heads_up.force_exit(player));
        }

        if let Some((cur_turn, bet_bound)) = bet_info {
            let _action = self.player_action(cur_turn, bet_bound).await;
        }

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
