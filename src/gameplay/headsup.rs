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
    Cash {
        buyin: CashBuyin,
        hands: Option<u16>,
    },
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Player {
    visibility: Visibility,
}

impl Player {
    pub async fn tick_event(&mut self) -> PlayerEvent {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Observer(Player);

impl Observer {
    pub async fn tick_event(&mut self) -> ObservableEvent {
        self.0.tick_event().await.unwrap_observable()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GameOver {
    Defeated(bool),
    ExitAbandon(bool),
    ExitCheckout(u32, u32),
    HandsReached(u32, u32),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Game;

impl Game {
    pub fn new(_game_type: GameType) -> (Self, Player, Player) {
        let player0 = Player {
            visibility: Visibility::Player0,
        };
        let player1 = Player {
            visibility: Visibility::Player1,
        };
        (Self, player0, player1)
    }

    pub fn observer(&mut self, visibility: Visibility) -> Option<Observer> {
        Some(Observer(Player { visibility }))
    }

    pub async fn run_hand(&mut self) -> Option<GameOver> {
        // Placeholder for the game logic
        // This function would contain the main game loop and logic for running a hand.
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
