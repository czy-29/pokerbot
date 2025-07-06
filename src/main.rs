use pokerbot::gameplay::{Deck, DisplayMode};

fn main() {
    // todo: DisplayConfig
    // default:
    //   - windows: ColoredUnicode (https://github.com/microsoft/terminal/issues/19100)
    //   - other: ColoredEmoji && !no_white (white canvas)

    // init:
    // todo: windows enable ANSI when `ColoredUnicode || (ColoredEmoji && !no_white)`
    // Only when `ColoredEmoji && !no_white`
    // print!("\x1b[107m\x1b[0J\x1b[30m");

    // todo: Hole/Flop/Board
    // note: 几种Display模式下的Hole/Flop/Board的间隔是不一样的

    let mut deck = Deck::default().shuffled();

    for (i, card) in deck.deal().enumerate() {
        match i % 5 {
            0 if i != 0 => println!(),
            3 | 4 => print!("  "),
            _ => (),
        }

        print!("{}  ", card.display(DisplayMode::ColoredEmoji));
    }

    deck.shuffle();
    println!();

    for (i, card) in deck.deal().enumerate() {
        match i % 5 {
            0 => println!(),
            3 | 4 => print!(" "),
            _ => (),
        }

        print!("{}  ", card.display(DisplayMode::ColoredUnicode));
    }

    deck.shuffle();
    println!();

    for (i, card) in deck.deal().enumerate() {
        match i % 5 {
            0 => println!(),
            3 | 4 => print!(" "),
            _ => (),
        }

        print!("{}  ", card.display(DisplayMode::Unicode));
    }

    deck.shuffle();
    println!();

    for (i, card) in deck.deal().enumerate() {
        match i % 5 {
            0 => println!(),
            3 | 4 => print!(" "),
            _ => (),
        }

        print!("{} ", card.display(DisplayMode::Ascii));
    }

    println!();

    // drop:
    // Only when `ColoredEmoji && !no_white`
    // print!("\x1b[0m\x1b[0J");
}
