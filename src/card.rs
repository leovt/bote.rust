use std::collections::HashMap;

/// A deck of cards lists the card definitions with their count
#[derive(Debug)]
pub struct Deck(pub Vec<(CardDefID, usize)>);

pub type CardDefID = usize;
/// A card definition consists of the mechanical and the display
/// cards with the same mechanics are the same for the purpose of the game
/// the display part is visual
#[derive(Debug)]
pub struct CardDefinition {
    pub id: CardDefID,
    pub mechanics: CardMechanics,
    pub display: CardDisplay,
}

#[derive(Debug)]
pub struct CardMechanics {
    pub is_token: bool,
    pub is_land: bool,
}

#[derive(Debug)]
pub struct CardDisplay();

pub type CardRepository = HashMap<CardDefID, CardDefinition>;
pub fn load_cards() -> CardRepository {
    HashMap::from([(
        101,
        CardDefinition {
            id: 101,
            mechanics: CardMechanics { is_token: false, is_land: true},
            display: CardDisplay(),
        },
    )])
}
