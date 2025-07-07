#![allow(dead_code)]

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
    HeroTurn,
    VillainTurn,
    Player0Turn,
    Player1Turn,
    GameOver(GameOver),
    GameAbort,
}

impl ObservableEvent {
    fn turn(player0: bool) -> Self {
        if player0 {
            Self::Player0Turn
        } else {
            Self::Player1Turn
        }
    }
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
    visibility: Visibility,
    game_over: Option<GameOver>,
    game_abort: bool,
    recv: UnboundedReceiver<PlayerEvent>,
}

impl Player {
    fn new(visibility: Visibility, recv: UnboundedReceiver<PlayerEvent>) -> Self {
        Self {
            visibility,
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
        // todo: transform event
        self.send.send(PlayerEvent::Observable(event)).is_ok()
    }

    async fn turn(&self, _bounds: ()) -> Option<ObservableEvent> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Game {
    game_over: Option<GameOver>,
    player0: PlayerSender,
    player1: PlayerSender,
    observer: Option<PlayerSender>,
}

impl Game {
    pub fn new(_game_type: GameType) -> (Self, Player, Player) {
        let p0_vis = Visibility::Player0;
        let p1_vis = Visibility::Player1;
        let (p0_send, p0_recv) = unbounded_channel();
        let (p1_send, p1_recv) = unbounded_channel();
        let game = Self {
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
        };
        let player0 = Player::new(p0_vis, p0_recv);
        let player1 = Player::new(p1_vis, p1_recv);
        (game, player0, player1)
    }

    pub fn observer(&mut self, visibility: Visibility) -> Option<Observer> {
        if self.observer.is_some() {
            return None; // Observer already exists
        }

        let (send, recv) = unbounded_channel();
        self.observer = Some(PlayerSender { visibility, send });
        Some(Observer(Player::new(visibility, recv)))
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
        let ob_event = ObservableEvent::turn(player0);
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

    pub async fn run_hand(&mut self) -> Option<GameOver> {
        if self.is_over() {
            return self.game_over();
        }

        let _button = true;
        let _big_blind = 500;
        let _stack0 = 150000;
        let _stack1 = 150000;
        let _exit_abandon = false;
        let _deck = 0;

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
