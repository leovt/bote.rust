/// A deck of cards lists the card definitions with their count
#[derive(Debug)]
struct Deck(Vec< (CardDefinition, usize) >);

/// A card definition consists of the mechanical and the display
/// cards with the same mechanics are the same for the purpose of the game
/// the display part is visual
#[derive(Debug)]
pub struct CardDefinition {
    mechanics: CardMechanics,
    display: CardDisplay
}

#[derive(Debug)]
struct CardMechanics {
    is_token: bool
}

#[derive(Debug)]
struct CardDisplay ();

pub static mountain:CardDefinition = CardDefinition { 
    mechanics: CardMechanics {is_token:false }, 
    display: CardDisplay () };
