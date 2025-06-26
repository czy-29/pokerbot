#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum CashBuyin {
    BB15,
    BB30,
    BB50,
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
    Cash(CashBuyin),
    SNG(SNGSpeed),
}
