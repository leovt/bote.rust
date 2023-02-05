#![allow(dead_code)]

use crate::card::CardDefinition;
mod card;

fn main() {
    println!("Hello, world!");
    let d1 = card::Deck(vec![(101, 10)]);
    let d2 = card::Deck(vec![(101, 10)]);
    let mut consumers: Vec<Box<dyn MessageConsumer>> = vec![Box::new(MessageLogger())];
    let card_repository = card::load_cards();
    let game = duel(User{name:"Leo".to_string()}, d1, 
                User{name:"Marc".to_string() }, d2, 
                &mut consumers,
                &card_repository
            );
    println!("{:?}", game);
}

trait MessageConsumer {
    fn handle_message(&mut self, _: &Message) -> Result<(), ()>;
}

struct MessageLogger ();

impl MessageConsumer for MessageLogger {
    fn handle_message(&mut self, msg: &Message) -> Result<(), ()> {
        println!("{:?}", msg);
        Ok(())
    }
}


#[derive(Debug)]
struct User{
    name : String
}

type CardID = usize;

#[derive(Debug)]
struct Card<'a> {
    id: CardID,
    owner_id: PlayerID,
    definition: &'a CardDefinition
}

type PlayerID = usize;
#[derive(Debug)]
struct Player<'a> {
    id : PlayerID,
    name : String,
    library : Vec<Card<'a>>,
    hand : Vec<Card<'a>>,
    graveyard : Vec<Card<'a>>
}

impl<'a> Player<'a> {
    fn new(id:PlayerID, name:String) -> Player<'a> {
        Player {id:id, name:name, library:Vec::new(), hand:Vec::new(), graveyard:Vec::new()}
    }
}

#[derive(Debug)]
struct Game<'a> {
    players : Vec<Player<'a>>,
    substep : Substep,
    step : Step,
    active_player_id : usize,
    priority_player_id : usize,
    card_repository : &'a card::CardRepository,
    next_id : usize
}

impl<'a> Game<'a> {
    fn new(card_repository: &'a card::CardRepository) -> Game {
        Game { 
            players: vec![], 
            substep: Substep::SetupGame, 
            step: Step::Untap,
            active_player_id: 0,
            priority_player_id: 0,
            card_repository: card_repository,
            next_id: 1001
        }
    }

    fn get_id(&mut self) -> usize {
        let res = self.next_id;
        self.next_id += 1;
        res
    }
}

#[derive(Debug)]
enum Message {
    CreatePlayer {id: PlayerID, name: String},
    AddCard {id: CardID, owner_id: PlayerID, def_id: card::CardDefID},
    Substep (Substep),
    Step (Step),
    BeginTurn (PlayerID),
    GetPriority (PlayerID)
}

impl<'a> MessageConsumer for Game<'a> {
    fn handle_message(&mut self, message: &Message) -> Result<(), ()> {
        match message {
            Message::CreatePlayer {id, name} => {
                if self.players.len() != *id {
                    Err(())
                } else {
                    let player = Player::new(*id, name.clone());
                    self.players.push(player);
                    Ok(())
                }
            }
            Message::AddCard { id, owner_id, def_id } => {
                match self.card_repository.get(def_id) {
                    Some(definition) => {
                        let card = Card {id:*id, owner_id:*owner_id, definition:definition};
                        self.players[*owner_id].library.push(card);
                        Ok(())},
                    None => Err(())
                }
            }
            Message::Substep (s) => {self.substep = *s; Ok(())}
            Message::Step (s) => {self.step = *s; Ok(())}
            Message::BeginTurn(pid) => {self.active_player_id = *pid; Ok(())}
            Message::GetPriority(pid) => {self.priority_player_id = *pid; Ok(())}
        }
    }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
enum Substep {
    SetupGame,
    CheckStateBasedActions,
    CheckTriggers,
    ResolveStack,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
enum Step {
    Untap, 
    Upkeep, 
    Draw,
    PrecombatMain,
    BeginCombat,
    DeclareBlockers,
    FirstStrikeDamage,
    SecondStrikeDamage,
    EndOfCombat,
    PostcombatMain,
    End,
    Cleanup
}

fn next_step(game: &Game) -> Vec<Message> {
    match game.substep {
        _ => vec![]
    }
}

fn duel<'a>(user1 : User, deck1 : card::Deck, user2 : User, deck2 : card::Deck, consumers: &mut Vec<Box<dyn MessageConsumer>>, card_repository: &'a card::CardRepository ) -> Game<'a> {
    let mut game = Game::new(card_repository);

    let mut init_msg = vec![
        Message::CreatePlayer{id:0, name:user1.name.clone()},
        Message::CreatePlayer{id:1, name:user2.name.clone()}
    ];

    for (def_id, count) in deck1.0 {
        for _ in 0..count {
            init_msg.push(Message::AddCard { id: game.get_id(), owner_id: 0, def_id: def_id})
        }
    }

    for (def_id, count) in deck2.0 {
        for _ in 0..count {
            init_msg.push(Message::AddCard { id: game.get_id(), owner_id: 1, def_id: def_id})
        }
    }

    for msg in &init_msg {
        game.handle_message(msg).unwrap();
        for consumer in &mut *consumers {
            let _ = consumer.handle_message(msg);
        }
    }

    game
}
