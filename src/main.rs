#![allow(dead_code)]

fn main() {
    println!("Hello, world!");
    let d1 = vec![Card(), Card(), Card()];
    let d2 = vec![Card(), Card(), Card(), Card()];
    let game = duel(User{name:"Leo".to_string()}, d1, 
                User{name:"Marc".to_string() }, d2);
    println!("{:?}", game);
}

trait MessageConsumer {
    fn handle_message(&mut self, _:Message) -> Result<(), ()>;
}

#[derive(Debug)]
struct User{
    name : String
}

/*
#[derive(Debug)]
struct MechanicCard{
    isToken : bool
}

#[derive(Debug)]
struct CardArt ();

#[derive(Debug)]
struct Card {
    mechanics : MechanicCard,
    art : CardArt
}

#[derive(Debug)]
struct GameCard {
    card: Card/* ,
    owner: &Player*/
}

*/

#[derive(Debug)]
struct Card ();

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
    fn handle_message(&mut self, message: Message) -> Result<(), ()> {
        match message{
            Message::CreatePlayer {id, name} => {
                if self.players.len() != id {
                    Err(())
                } else {
                    let player = Player {id:id, name:name};
                    self.players.push(player);
                    Ok(())
                }
            }
            Message::Substep (s) => {self.substep = s; Ok(())}
            Message::Step (s) => {self.step = s; Ok(())}
            Message::BeginTurn(pid) => {self.active_player_id = pid; Ok(())}
            Message::GetPriority(pid) => {self.priority_player_id = pid; Ok(())}
        }
    }
}

#[derive(Debug)]
enum Substep {
    SetupGame,
    CheckStateBasedActions,
    CheckTriggers,
    ResolveStack,
}

#[derive(Debug)]
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
    vec![]
}

fn duel(user1 : User, deck1 : Vec<Card>, user2 : User, deck2 : Vec<Card>) -> Game {
    let mut game = Game { 
        players: vec![], 
        substep: Substep::SetupGame, 
        step: Step::Untap,
        active_player_id: 0,
        priority_player_id: 0
    };
    game.handle_message(Message::CreatePlayer{id:0, name:user1.name.clone()}).expect("Error creating Player 1");
    game.handle_message(Message::CreatePlayer{id:1, name:user2.name.clone()}).expect("Error creating Player 2");
    game
}

struct Permanent {
    card: Card
}