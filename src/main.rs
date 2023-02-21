#![allow(dead_code)]

use crate::card::CardDefinition;
mod card;
use rand::seq::SliceRandom; // Vec.shuffle
use rand::thread_rng;

fn main() {
    let d1 = card::Deck(vec![(101, 10)]);
    let d2 = card::Deck(vec![(101, 10)]);
    let mut consumers: Vec<Box<dyn MessageConsumer>> = vec![Box::new(MessageLogger())];
    let card_repository = card::load_cards();
    let game = duel(
        User {
            name: "Leo".to_string(),
        },
        d1,
        User {
            name: "Marc".to_string(),
        },
        d2,
        &mut consumers,
        &card_repository,
    );
    println!("{:?}", game);
}

trait MessageConsumer {
    fn handle_message(&mut self, _: &Message) -> Result<(), HandleError>;
}

struct MessageLogger();

impl MessageConsumer for MessageLogger {
    fn handle_message(&mut self, msg: &Message) -> Result<(), HandleError> {
        println!("{:?}", msg);
        Ok(())
    }
}

#[derive(Debug)]
struct User {
    name: String,
}

type CardID = usize;

#[derive(Debug)]
struct Card<'a> {
    id: CardID,
    owner_id: PlayerID,
    definition: &'a CardDefinition,
}

type PlayerID = usize;
#[derive(Debug)]
struct Player<'a> {
    id: PlayerID,
    name: String,
    library: Vec<Card<'a>>,
    hand: Vec<Card<'a>>,
    graveyard: Vec<Card<'a>>,
    has_drawn_from_empty: bool,
    has_lost: bool,
    has_passed: bool,
    life: i32,
}

impl<'a> Player<'a> {
    fn new(id: PlayerID, name: String) -> Player<'a> {
        Player {
            id: id,
            name: name,
            library: Vec::new(),
            hand: Vec::new(),
            graveyard: Vec::new(),
            has_drawn_from_empty: false,
            has_lost: false,
            has_passed: false,
            life: 20,
        }
    }

    fn max_hand_size(&self) -> i32 {
        7
    }
}

type SpellID = usize;
#[derive(Debug)]
struct Spell {
    id: SpellID,
}
impl Spell {
    fn resolve(&self) -> Vec<Message> {
        Vec::new()
    }
}

#[derive(Debug)]
struct Game<'a> {
    players: Vec<Player<'a>>,
    substep: Substep,
    step: Step,
    active_player_id: usize,
    priority_player_id: usize,
    card_repository: &'a card::CardRepository,
    stack: Vec<Spell>,
    battlefield: Vec<()>,
    attackers: Vec<()>,
    next_id: usize,
}

impl<'a> Game<'a> {
    fn new(card_repository: &'a card::CardRepository) -> Game {
        Game {
            players: vec![],
            substep: Substep::InitialShuffle,
            step: Step::Untap,
            active_player_id: 0,
            priority_player_id: 0,
            card_repository: card_repository,
            stack: Vec::new(),
            battlefield: Vec::new(),
            attackers: Vec::new(),
            next_id: 1001,
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
    CreatePlayer {
        id: PlayerID,
        name: String,
    },
    AddCard {
        id: CardID,
        owner_id: PlayerID,
        def_id: card::CardDefID,
    },
    Substep(Substep),
    Step(Step),
    BeginTurn(PlayerID),
    GetPriority(PlayerID),
    ShuffleLibrary(PlayerID),
    DrawCard(PlayerID, CardID),
    DrawFromEmpty(PlayerID),
    PlayerLoses(PlayerID),
    PlayerWins(PlayerID),
    PlayerHasPriority(PlayerID),
    PlayerPasses(PlayerID),
    PriorityEnded,
    ResolveSpell(SpellID),
}

#[derive(Debug)]
enum HandleError {
    PlayerIdError,
    CardIdError,
    CardDefIdError,
}

impl<'a> MessageConsumer for Game<'a> {
    fn handle_message(&mut self, message: &Message) -> Result<(), HandleError> {
        match message {
            Message::CreatePlayer { id, name } => {
                if self.players.len() != *id {
                    Err(HandleError::PlayerIdError)
                } else {
                    let player = Player::new(*id, name.clone());
                    self.players.push(player);
                    Ok(())
                }
            }
            Message::AddCard {
                id,
                owner_id,
                def_id,
            } => match self.card_repository.get(def_id) {
                Some(definition) => {
                    let card = Card {
                        id: *id,
                        owner_id: *owner_id,
                        definition: definition,
                    };
                    self.players[*owner_id].library.push(card);
                    Ok(())
                }
                None => Err(HandleError::CardDefIdError),
            },
            Message::Substep(s) => {
                self.substep = *s;
                Ok(())
            }
            Message::Step(s) => {
                self.step = *s;
                Ok(())
            }
            Message::BeginTurn(pid) => {
                self.active_player_id = *pid;
                Ok(())
            }
            Message::GetPriority(pid) => {
                self.priority_player_id = *pid;
                Ok(())
            }
            Message::ShuffleLibrary(pid) => {
                self.players[*pid].library.shuffle(&mut thread_rng());
                Ok(())
            }
            Message::DrawCard(pid, cid) => match self.players[*pid].library.pop() {
                Some(card) => {
                    if card.id == *cid {
                        self.players[*pid].hand.push(card);
                        Ok(())
                    } else {
                        Err(HandleError::CardIdError)
                    }
                }
                None => Err(HandleError::CardIdError),
            },
            Message::DrawFromEmpty(pid) => {
                self.players[*pid].has_drawn_from_empty = true;
                Ok(())
            }
            Message::PlayerLoses(pid) => {
                self.players[*pid].has_lost = true;
                Ok(())
            }
            Message::PlayerWins(_pid) => {
                // Do Nothing
                Ok(())
            }
            Message::PlayerHasPriority(pid) => {
                self.priority_player_id = *pid;
                Ok(())
            }
            Message::PlayerPasses(pid) => {
                self.players[*pid].has_passed = true;
                Ok(())
            }
            Message::PriorityEnded => {
                for p in self.players.iter_mut() {
                    p.has_passed = false;
                }
                Ok(())
            }
            Message::ResolveSpell(sid) => match self.stack.pop() {
                Some(spell) if spell.id == *sid => Ok(()),
                _ => Err(HandleError::CardIdError),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Substep {
    InitialShuffle,
    InitialDrawCards,
    BeginOfStep,
    CheckStateBasedActions,
    CheckTriggers,
    PlayerPriority,
    ResolveStack,
    EndOfStep,
    GameEnded,
}

#[derive(Debug, Clone, Copy)]
enum Step {
    Untap,
    Upkeep,
    Draw,
    PrecombatMain,
    BeginCombat,
    DeclareAttackers,
    DeclareBlockers,
    FirstStrikeDamage,
    SecondStrikeDamage,
    EndOfCombat,
    PostcombatMain,
    End,
    Cleanup,
}

fn try_draw_cards(player: &Player, count: usize) -> Vec<Message> {
    let mut msg = Vec::new();
    let n = player.library.len();
    for i in 0..count {
        if i < n {
            let card = &player.library[n - 1 - i];
            msg.push(Message::DrawCard(player.id, card.id));
        } else {
            msg.push(Message::DrawFromEmpty(player.id));
            break;
        }
    }
    msg
}

fn try_draw_card(player: &Player) -> Message {
    match player.library.last() {
        Some(card) => Message::DrawCard(player.id, card.id),
        None => Message::DrawFromEmpty(player.id),
    }
}

fn state_based_actions(game: &Game) -> Vec<Message> {
    let mut msg = Vec::new();
    for player in &game.players {
        if player.life <= 0 || player.has_drawn_from_empty {
            msg.push(Message::PlayerLoses(player.id));
        }
    }
    // TODO: put all creatures whose damage exceeds their toughness into the graveyard
    // TODO: put unattached enchantements into the graveyard
    // Note: Contrary to magic the gathering winning is also a state based action
    let nb_losing_players = game.players.iter().filter(|p| p.has_lost).count();
    if nb_losing_players == game.players.len() {
        // all have lost
        msg.push(Message::Substep(Substep::GameEnded));
    } else if nb_losing_players == game.players.len() - 1 {
        let winner = game
            .players
            .iter()
            .filter(|p| !p.has_lost)
            .next()
            .expect("there must be a winner");
        msg.push(Message::PlayerWins(winner.id));
        msg.push(Message::Substep(Substep::GameEnded));
    }
    msg
}

fn start_player_priority(game: &Game) -> Vec<Message> {
    let mut msg = Vec::new();
    assert!(game.players.iter().all(|p| !p.has_passed));
    msg.push(Message::Substep(Substep::PlayerPriority));
    msg.push(Message::PlayerHasPriority(game.active_player_id));
    msg
}

fn next_step(game: &Game) -> Vec<Message> {
    let mut msg = Vec::new();
    match game.substep {
        Substep::InitialShuffle => {
            for player in &game.players {
                msg.push(Message::ShuffleLibrary(player.id));
            }
            msg.push(Message::Substep(Substep::InitialDrawCards));
        }
        Substep::InitialDrawCards => {
            for player in &game.players {
                msg.extend(try_draw_cards(player, 7))
            }
            msg.push(Message::Substep(Substep::CheckStateBasedActions));
        }
        Substep::CheckStateBasedActions => {
            let actions = state_based_actions(game);
            if actions.len() > 0 {
                msg.extend(actions);
            } else {
                msg.extend(start_player_priority(game));
            }
        }
        Substep::PlayerPriority => {
            let priority_player = &game.players[game.priority_player_id];
            if priority_player.has_passed {
                let next_player_id = (game.priority_player_id + 1) % game.players.len();
                if next_player_id == game.active_player_id {
                    assert!(game.players.iter().all(|p| p.has_passed));
                    msg.push(Message::PriorityEnded);
                    msg.push(Message::Substep(Substep::ResolveStack))
                } else {
                    msg.push(Message::PlayerHasPriority(next_player_id));
                }
            } else {
                // todo: ask player what they want to do
                msg.push(Message::PlayerPasses(priority_player.id));
            }
        }
        Substep::ResolveStack => match game.stack.last() {
            Some(spell) => {
                msg.push(Message::ResolveSpell(spell.id));
                msg.extend(spell.resolve());
            }
            None => {
                msg.push(Message::Substep(Substep::EndOfStep));
            }
        },
        Substep::EndOfStep => {
            // todo: empty mana pool
            use Step::*;
            msg.push(Message::Step(match game.step {
                Untap => Upkeep,
                Upkeep => Draw,
                Draw => PrecombatMain,
                PrecombatMain => BeginCombat,
                BeginCombat => DeclareAttackers,
                DeclareAttackers => {
                    if game.attackers.len() > 0 {
                        DeclareBlockers
                    } else {
                        EndOfCombat
                    }
                }
                DeclareBlockers => FirstStrikeDamage,
                FirstStrikeDamage => SecondStrikeDamage,
                SecondStrikeDamage => EndOfCombat,
                EndOfCombat => PostcombatMain,
                PostcombatMain => End,
                End => Cleanup,
                Cleanup => Untap,
            }));
            msg.push(Message::Substep(Substep::BeginOfStep));
        }
        Substep::BeginOfStep => {
            match game.step {
                Step::Untap => {
                    // Untap all permanents controlled by active player
                    msg.push(Message::Substep(Substep::CheckStateBasedActions));
                }
                Step::Draw => {
                    let player = &game.players[game.active_player_id];
                    msg.extend(try_draw_cards(player, 1));
                    msg.push(Message::Substep(Substep::CheckStateBasedActions));
                }
                Step::Cleanup => {
                    let player = &game.players[game.active_player_id];
                    let number_to_discard = player.hand.len() as i32 - player.max_hand_size();
                    if number_to_discard > 0 {
                        // msg.extend(Message::QueryPlayerAction(Action::Discard())
                        msg.push(Message::Substep(Substep::EndOfStep));
                    } else {
                        // all damage is removed
                        // until end of turn ends
                        msg.push(Message::Substep(Substep::EndOfStep));
                    }
                }
                _ => msg.push(Message::Substep(Substep::CheckStateBasedActions))
            }
        }
        _ => {
            msg.push(Message::Substep(Substep::GameEnded));
        }
    };
    msg
}

fn duel<'a>(
    user1: User,
    deck1: card::Deck,
    user2: User,
    deck2: card::Deck,
    consumers: &mut Vec<Box<dyn MessageConsumer>>,
    card_repository: &'a card::CardRepository,
) -> Game<'a> {
    let mut game = Game::new(card_repository);

    let mut init_msg = vec![
        Message::CreatePlayer {
            id: 0,
            name: user1.name.clone(),
        },
        Message::CreatePlayer {
            id: 1,
            name: user2.name.clone(),
        },
    ];

    for (def_id, count) in deck1.0 {
        for _ in 0..count {
            init_msg.push(Message::AddCard {
                id: game.get_id(),
                owner_id: 0,
                def_id: def_id,
            })
        }
    }

    for (def_id, count) in deck2.0 {
        for _ in 0..count {
            init_msg.push(Message::AddCard {
                id: game.get_id(),
                owner_id: 1,
                def_id: def_id,
            })
        }
    }

    for msg in &init_msg {
        game.handle_message(msg).unwrap();
        for consumer in &mut *consumers {
            let _ = consumer.handle_message(msg);
        }
    }

    while game.substep != Substep::GameEnded {
        for msg in next_step(&game) {
            game.handle_message(&msg).unwrap();
            for consumer in &mut *consumers {
                let _ = consumer.handle_message(&msg);
            }
        }
    }
    game
}
