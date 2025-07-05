use pokerbot::gameplay::{Deck, DisplayMode};

fn main() {
    let mut deck = Deck::default().shuffled();

    for (i, card) in deck.deal().enumerate() {
        match i % 5 {
            0 if i != 0 => println!(),
            3 | 4 => print!("  "),
            _ => (),
        }

        print!("{}  ", card.display(DisplayMode::Emoji));
    }

    deck.shuffle();
    println!();

    for (i, card) in deck.deal().enumerate() {
        match i % 5 {
            0 => println!(),
            3 | 4 => print!(" "),
            _ => (),
        }

        print!("{} ", card.display(DisplayMode::Unicode));
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
}
