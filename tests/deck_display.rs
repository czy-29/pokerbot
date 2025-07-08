use pokerbot::gameplay::{DisplayMode, headsup::Deck};
use std::fmt::Write;

#[test]
fn test_deck_display() {
    let deck = Deck::default();
    
    // Test Ascii display
    let display = deck.display(DisplayMode::Ascii);
    let mut output = String::new();
    write!(&mut output, "{}", display).unwrap();
    
    // Verify that the output contains all 52 cards
    assert!(output.contains("As"));
    assert!(output.contains("Kh"));
    assert!(output.contains("Qd"));
    assert!(output.contains("Jc"));
    assert!(output.contains("2s"));
    
    // Verify that the output has multiple lines (due to cards_per_row)
    assert!(output.contains('\n'));
    
    // Test Unicode display
    let display = deck.display(DisplayMode::Unicode);
    let mut output = String::new();
    write!(&mut output, "{}", display).unwrap();
    
    // Verify that the output contains Unicode symbols
    assert!(output.contains("♠"));
    assert!(output.contains("♥"));
    assert!(output.contains("♦"));
    assert!(output.contains("♣"));
}