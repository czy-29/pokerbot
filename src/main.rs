use pokerbot::gameplay::Deck;

fn main() {
    let mut deck = Deck::default().shuffled();

    for (i, card) in deck.deal().enumerate() {
        if i % 5 == 0 && i != 0 {
            println!();
        }

        print!("{} ", card.display_unicode());
    }

    deck.shuffle();
    println!();

    for (i, card) in deck.deal().enumerate() {
        if i % 5 == 0 {
            println!();
        }

        print!("{} ", card.display_ascii());
    }
}
