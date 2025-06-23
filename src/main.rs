use pokerbot::gameplay::Deck;

fn main() {
    let deck = Deck::default();

    for (i, card) in deck.deal().enumerate() {
        if i % 4 == 0 && i != 0 {
            println!();
        }

        print!("{} ", card.display_unicode());
    }

    println!();

    for (i, card) in deck.deal().enumerate() {
        if i % 4 == 0 {
            println!();
        }

        print!("{} ", card.display_ascii());
    }
}
