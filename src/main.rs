use pokerbot::gameplay::{DisplayMode, headsup::Deck};

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

    // Display entire deck using the new display trait
    println!("Deck display (ColoredEmoji):");
    println!("{}", deck.display(DisplayMode::ColoredEmoji));
    println!();

    // Original card-by-card display
    println!("Card-by-card display (ColoredEmoji):");
    for (i, card) in deck.deal().enumerate() {
        match i % 5 {
            0 if i != 0 => println!(),
            3 | 4 => print!("  "),
            _ => (),
        }

        print!("{}  ", card.display(DisplayMode::ColoredEmoji));
    }

    deck.shuffle();
    println!("\n");

    println!("Deck display (ColoredUnicode):");
    println!("{}", deck.display(DisplayMode::ColoredUnicode));
    println!();

    println!("Card-by-card display (ColoredUnicode):");
    for (i, card) in deck.deal().enumerate() {
        match i % 5 {
            0 => println!(),
            3 | 4 => print!(" "),
            _ => (),
        }

        print!("{}  ", card.display(DisplayMode::ColoredUnicode));
    }

    deck.shuffle();
    println!("\n");

    println!("Deck display (Unicode):");
    println!("{}", deck.display(DisplayMode::Unicode));
    println!();

    println!("Card-by-card display (Unicode):");
    for (i, card) in deck.deal().enumerate() {
        match i % 5 {
            0 => println!(),
            3 | 4 => print!(" "),
            _ => (),
        }

        print!("{}  ", card.display(DisplayMode::Unicode));
    }

    deck.shuffle();
    println!("\n");

    println!("Deck display (Ascii):");
    println!("{}", deck.display(DisplayMode::Ascii));
    println!();

    println!("Card-by-card display (Ascii):");
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
