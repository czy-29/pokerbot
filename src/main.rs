use pokerbot::gameplay::{Deck, DisplayMode};

fn main() {
    // init:
    // todo: windows enable ANSI when `ColoredUnicode || (ColoredEmoji && white)`
    // Only when `ColoredEmoji && white`
    // print!("\x1b[107m\x1b[0J\x1b[30m");

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
    // Only when `ColoredEmoji && white`
    // print!("\x1b[0m\x1b[0J");
}
