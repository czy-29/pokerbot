use pokerbot::gameplay::{
    Board, DisplayMode,
    headsup::{Dealer, Deck},
};

fn display(mut dealer: Dealer, mode: DisplayMode) {
    println!("{}", dealer.deal_hole().display(mode));
    println!("{}", dealer.deal_hole().display(mode));

    let mut board = Board::flop(dealer.deal_flop());
    println!("{}", board.display(mode));

    board = board.turn(dealer.deal_card()).unwrap();
    println!("{}", board.display(mode));

    board = board.river(dealer.deal_card()).unwrap();
    println!("{}", board.display(mode));
}

fn main() {
    // todo: DisplayConfig
    // default:
    //   - windows: ColoredUnicode (https://github.com/microsoft/terminal/issues/19100)
    //   - other: ColoredEmoji && !no_white (white canvas)

    // init:
    // todo: windows enable ANSI when `ColoredUnicode || (ColoredEmoji && !no_white)`
    // Only when `ColoredEmoji && !no_white`:
    // print!("\x1b[107m\x1b[0J\x1b[30m");

    let mut deck = Deck::default();
    display(deck.shuffle_and_deal(), DisplayMode::ColoredEmoji);

    println!();
    display(deck.shuffle_and_deal(), DisplayMode::ColoredUnicode);

    println!();
    display(deck.shuffle_and_deal(), DisplayMode::Unicode);

    println!();
    display(deck.shuffle_and_deal(), DisplayMode::Ascii);

    // drop:
    // Only when `ColoredEmoji && !no_white`:
    // print!("\x1b[0m\x1b[0J");
}
