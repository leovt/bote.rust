#![allow(dead_code)]

use crate::card::CardDefinition;
mod card;

fn main() {
    println!("Hello, world!");
    let d1 = vec![
        Card { id:0, owner_id:0, definition: &card::mountain},
        Card { id:1, owner_id:0, definition: &card::mountain},
        Card { id:2, owner_id:0, definition: &card::mountain}];
    let d2 = vec![
        Card { id:3, owner_id:1, definition: &card::mountain},
        Card { id:4, owner_id:1, definition: &card::mountain}];
    let mut consumers: Vec<Box<dyn MessageConsumer>> = vec![Box::new(MessageLogger())];
    let game = duel(User{name:"Leo".to_string()}, d1, 
                User{name:"Marc".to_string() }, d2, &mut consumers);
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
struct Player {
    id : PlayerID,
    name : String
 /*   library : Vec<Card>,
    hand : Vec<Card>,
    graveyard : Vec<Card>,*/
}

#[derive(Debug)]
struct Game {
    players : Vec<Player>,
    substep : Substep,
    step : Step,
    active_player_id : usize,
    priority_player_id : usize,
}

#[derive(Debug)]
enum Message {
    CreatePlayer {id: PlayerID, name: String},
    Substep (Substep),
    Step (Step),
    BeginTurn (PlayerID),
    GetPriority (PlayerID)
}

impl MessageConsumer for Game {
    fn handle_message(&mut self, message: &Message) -> Result<(), ()> {
        match message {
            Message::CreatePlayer {id, name} => {
                if self.players.len() != *id {
                    Err(())
                } else {
                    let player = Player {id:*id, name:name.clone()};
                    self.players.push(player);
                    Ok(())
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

fn duel(user1 : User, deck1 : Vec<Card>, user2 : User, deck2 : Vec<Card>, consumers: &mut Vec<Box<dyn MessageConsumer>>) -> Game {
    let mut game = Game { 
        players: vec![], 
        substep: Substep::SetupGame, 
        step: Step::Untap,
        active_player_id: 0,
        priority_player_id: 0
    };

    let init_msg = vec![
        Message::CreatePlayer{id:0, name:user1.name.clone()},
        Message::CreatePlayer{id:1, name:user2.name.clone()}
    ];

    for msg in &init_msg {
        game.handle_message(msg).unwrap();
        for consumer in &mut *consumers {
            let _ = consumer.handle_message(msg);
        }
    }

    game
}
