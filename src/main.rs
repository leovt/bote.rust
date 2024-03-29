#![allow(dead_code)]

use crate::card::CardDefinition;
//use crate::energy::Energy;
mod card;
//mod energy;
use rand::seq::SliceRandom; // Vec.shuffle
use rand::thread_rng;

use std::collections::HashMap;

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

#[derive(Debug, Clone, PartialEq)]
enum PriorityAction {
    Pass,
    PlayLand(CardID),
}

#[derive(Debug, Clone)]
enum Query {
    Discard(Vec<CardID>, i32),
    PriorityAction(Vec<PriorityAction>),
}

#[derive(Debug)]
enum Answer {
    Discard(Vec<CardID>),
    PriorityAction(PriorityAction),
}

fn validate_answer(query: &Query, answer: &Answer) -> bool {
    match (query, answer) {
        (Query::Discard(cards, n), Answer::Discard(selected)) => {
            selected.len() as i32 == *n && selected.iter().all(|c| cards.contains(c))
        }
        (Query::PriorityAction(actions), Answer::PriorityAction(action)) => {
            actions.contains(action)
        }
        _ => false,
    }
}

fn random_answer(query: &Query) -> Answer {
    let answer = match query {
        Query::Discard(cards, n) => Answer::Discard(
            cards
                .choose_multiple(&mut thread_rng(), *n as usize)
                .cloned()
                .collect::<Vec<CardID>>(),
        ),
        Query::PriorityAction(actions) => Answer::PriorityAction(
            actions
                .choose(&mut thread_rng())
                .expect("malformed query")
                .clone(),
        ),
    };
    assert!(validate_answer(query, &answer));
    answer
}

#[derive(Debug)]
struct User {
    name: String,
}

type CardID = usize;
type PublicCardID = usize;

#[derive(Debug)]
struct Card<'a> {
    id: CardID,
    public_id: PublicCardID,
    object_id: Option<ObjectID>,
    owner_id: PlayerID,
    definition: &'a CardDefinition,
}

type ObjectID = usize;

#[derive(Debug)]
struct Object {
    // controller
    id: ObjectID,
    kind: ObjectKind,
    location: ObjectLocation,
}

#[derive(Debug)]
enum ObjectLocation {
    Library,
    Hand,
    Stack,
    Battlefield,
    Graveyard,
}

#[derive(Debug)]
enum ObjectKind {
    Card(CardID),
    ActivatedAbility(CardID, usize),
}

type PlayerID = usize;
#[derive(Debug)]
struct Player {
    id: PlayerID,
    name: String,
    library: Vec<CardID>,
    hand: Vec<CardID>,
    graveyard: Vec<CardID>,
    has_drawn_from_empty: bool,
    has_lost: bool,
    has_passed: bool,
    life: i32,
    lands_played: u32,
}

impl Player {
    fn new(id: PlayerID, name: String) -> Player {
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
            lands_played: 0,
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
    cards: HashMap<CardID, Card<'a>>,
    players: Vec<Player>,
    substep: Substep,
    step: Step,
    active_player_id: usize,
    priority_player_id: usize,
    card_repository: &'a card::CardRepository,
    stack: Vec<Spell>,
    maybe_query: Option<Query>,
    maybe_answer: Option<Answer>,
    next_id: usize,
    objects: HashMap<ObjectID, Object>,
}

impl<'a> Game<'a> {
    fn new(card_repository: &'a card::CardRepository) -> Game {
        Game {
            cards: HashMap::new(),
            players: Vec::new(),
            substep: Substep::InitialShuffle,
            step: Step::Untap,
            active_player_id: 0,
            priority_player_id: 0,
            card_repository: card_repository,
            stack: Vec::new(),
            next_id: 1001,
            objects: HashMap::new(),
            maybe_answer: None,
            maybe_query: None,
        }
    }

    fn get_id(&mut self) -> usize {
        let res = self.next_id;
        self.next_id += 1;
        res
    }

    fn commit_id(&mut self, id: usize) {
        assert!(id >= self.next_id);
        self.next_id = id;
    }
}

#[derive(Debug)]
enum Message {
    Query(Query),
    RejectAnswer,
    AcceptAnswer,
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
    Discard(PlayerID, CardID),
    PlayLand(PlayerID, CardID, ObjectID),
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
            Message::Query(query) => {
                self.maybe_query = Some(query.clone());
                self.maybe_answer = None;
                Ok(())
            }
            Message::AcceptAnswer => {
                self.maybe_query = None;
                self.maybe_answer = None;
                Ok(())
            }
            Message::RejectAnswer => {
                self.maybe_query = None;
                self.maybe_answer = None;
                Ok(())
            }
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
                        public_id: 0,
                        owner_id: *owner_id,
                        object_id: None,
                        definition: definition,
                    };
                    self.cards.insert(*id, card);
                    self.players[*owner_id].library.push(*id);
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
                self.players[*pid].lands_played = 0;
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
                Some(card_id) => {
                    if card_id == *cid {
                        self.players[*pid].hand.push(card_id);
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
            Message::Discard(pid, cid) => {
                if let Some(i) = self.players[*pid].hand.iter().position(|c| *c == *cid) {
                    let card_id = self.players[*pid].hand.remove(i);
                    self.players[*pid].graveyard.push(card_id);
                    Ok(())
                } else {
                    Err(HandleError::CardIdError)
                }
            }
            Message::PlayLand(pid, cid, oid) => {
                if let Some(i) = self.players[*pid].hand.iter().position(|c| *c == *cid) {
                    let card_id = self.players[*pid].hand.remove(i);
                    self.commit_id(*oid);
                    let object = Object {
                        id: *oid,
                        kind: ObjectKind::Card(card_id),
                        location: ObjectLocation::Battlefield,
                    };
                    self.players[*pid].lands_played += 1;
                    self.objects.insert(*oid, object);
                    Ok(())
                } else {
                    Err(HandleError::CardIdError)
                }
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            let card_id = player.library[n - 1 - i];
            msg.push(Message::DrawCard(player.id, card_id));
        } else {
            msg.push(Message::DrawFromEmpty(player.id));
            break;
        }
    }
    msg
}

fn try_draw_card(player: &Player) -> Message {
    match player.library.last() {
        Some(card_id) => Message::DrawCard(player.id, *card_id),
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

fn ask_query<'a>(game: &'a Game<'a>, msg: &mut Vec<Message>, query: Query) -> Option<&'a Answer> {
    match &game.maybe_answer {
        Some(answer) if validate_answer(&query, answer) => {
            msg.push(Message::AcceptAnswer);
            Some(answer)
        }
        Some(_) => {
            msg.push(Message::RejectAnswer);
            msg.push(Message::Query(query));
            None
        }
        _ => {
            msg.push(Message::Query(query));
            None
        }
    }
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
                let mut actions = vec![PriorityAction::Pass];
                if (game.step == Step::PrecombatMain || game.step == Step::PostcombatMain)
                    && game.active_player_id == game.priority_player_id
                {
                    if priority_player.lands_played < 1 {
                        for card_id in priority_player.hand.iter() {
                            if game.cards[card_id].definition.mechanics.is_land {
                                actions.push(PriorityAction::PlayLand(*card_id));
                            }
                        }
                    }
                }
                let query = Query::PriorityAction(actions);
                if let Some(Answer::PriorityAction(action)) = ask_query(game, &mut msg, query) {
                    match action {
                        PriorityAction::Pass => msg.push(Message::PlayerPasses(priority_player.id)),
                        PriorityAction::PlayLand(cid) => msg.push(Message::PlayLand(
                            game.priority_player_id,
                            *cid,
                            game.next_id,
                        )),
                    }
                }
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
            if game.step == Cleanup {
                msg.push(Message::BeginTurn(
                    (game.active_player_id + 1) % game.players.len(),
                ));
            }
            msg.push(Message::Step(match game.step {
                Untap => Upkeep,
                Upkeep => Draw,
                Draw => PrecombatMain,
                PrecombatMain => BeginCombat,
                BeginCombat => DeclareAttackers,
                DeclareAttackers => {
                    if false
                    /* attackers declared ? */
                    {
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
                        let query = Query::Discard(player.hand.clone(), number_to_discard);
                        if let Some(Answer::Discard(card_ids)) = ask_query(&game, &mut msg, query) {
                            for card_id in card_ids {
                                msg.push(Message::Discard(player.id, *card_id));
                            }
                            msg.push(Message::Substep(Substep::EndOfStep));
                        }
                    } else {
                        // all damage is removed
                        // until end of turn ends
                        msg.push(Message::Substep(Substep::EndOfStep));
                    }
                }
                _ => msg.push(Message::Substep(Substep::CheckStateBasedActions)),
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
            match &msg {
                Message::Query(query) => game.maybe_answer = Some(random_answer(&query)),
                _ => (),
            }
            for consumer in &mut *consumers {
                let _ = consumer.handle_message(&msg);
            }
        }
    }
    game
}
