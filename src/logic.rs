use crate::card::{ Card, Suit, Value };
use crate::renderer::{CardLocation, CardTarget};

use rand::seq::SliceRandom;
use rand::rng;

// definitions ---

#[derive(Debug)]
pub enum GameError {
    PileFull,
    InvalidMove(&'static str),
    WrongSuit,
    Unknown,
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::PileFull => write!(f, "The selected pile is full"),
            GameError::InvalidMove(msg) => write!(f, "That is an invalid Solitaire move: {}", msg),
            GameError::WrongSuit => write!(f, "The card suit does not match the pile requirement"),
            GameError::Unknown => write!(f, "An unknown game error occurred"),
        }
    }
}

impl std::error::Error for GameError {}

impl From<GameError> for std::io::Error {
    fn from(error: GameError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, error)
    }
}

pub struct Tableau {
    pub cols: [Vec<Card>; 7],
}

pub struct Foundation {
    pub piles: [(Vec<Card>, bool); 4], // (suit, is_complete)
}

pub struct Stock {
    pub cards: Vec<Card>,
}

pub struct Klondike {
    pub tableau: Tableau,
    pub foundation: Foundation,
    pub stock: Stock,

    pub cards_in_play: (Vec<Card>, CardTarget), // (cards, original target)
}

// impls ---

impl Tableau {
    pub fn new(stock: &mut Stock) -> Self {
        let mut cols: [Vec<Card>; 7] = Default::default();

        for i in 0..7 {
            let card_count = i + 1;
            cols[i] = stock.take_n(card_count);
            cols[i].last_mut().map(|c| {
                c.face_up = true;
            });
        }

        Self { cols }
    }

    pub fn get_valid_cards_from(&mut self, target: CardTarget) -> Option<Vec<Card>> {
        let col = self.cols.get_mut(target.col_idx)?;

        let clicked_card = col.get(target.card_idx)?;

        if !clicked_card.face_up {
            return None;
        }

        let moving_cards = col.split_off(target.card_idx);

        Some(moving_cards)
    }

    pub fn place_cards_in_col(&mut self, cards: &Vec<Card>, col_idx: usize) -> Result<(), GameError> {
        if cards.is_empty() {
            return Err(GameError::InvalidMove("No cards to place"));
        }

        // empty col
        let selected_col = &mut self.cols[col_idx];
        if selected_col.is_empty() && cards[0].value != Value::King {
            return Err(GameError::InvalidMove("Only Kings can be placed in an empty column"));
        }

        // check suit/color
        let last_col_card = selected_col.last().expect("somehow col is empty");
        match cards[0].suit {
            Suit::Hearts | Suit::Diamonds => {
                if last_col_card.suit != Suit::Spades && last_col_card.suit != Suit::Clubs {
                    return Err(GameError::InvalidMove("Invalid suit for the target column"));
                }
            },
            Suit::Clubs | Suit::Spades => {
                if last_col_card.suit != Suit::Hearts && last_col_card.suit != Suit::Diamonds {
                    return Err(GameError::InvalidMove("Invalid suit for the target column"));
                }
            },
        }

        // check value
        if cards[0].value as usize != (last_col_card.value as usize) - 1 {
            return Err(GameError::InvalidMove("Invalid value for the target column"));
        }

        self.cols[col_idx].extend(cards);
        Ok(())
    }
}

impl Foundation {
    pub fn new() -> Self {
        Self {
            piles: [
                (Vec::with_capacity(13), false), // hearts
                (Vec::with_capacity(13), false), // diamonds
                (Vec::with_capacity(13), false), // clubs
                (Vec::with_capacity(13), false), // spades
            ],
        }
    }

    pub fn place_card_on_pile(&mut self, card: Card, selected_pile: Suit) -> Result<(), GameError> {
        if card.suit != selected_pile {
            return Err(GameError::WrongSuit);
        }

        let pile = &mut self.piles[selected_pile as usize];
        if pile.0.is_empty() && card.value != Value::Ace {
            return Err(GameError::InvalidMove("Only Aces can be placed on an empty foundation pile"));
        }

        if
            !pile.0.is_empty() &&
            (card.value as usize) != (pile.0.last().unwrap().value as usize) + 1
        {
            return Err(GameError::InvalidMove("Invalid card placement"));
        }

        pile.0.push(card);
        pile.1 = card.value == Value::King;
        return Ok(());
    }

    pub fn take_card_from_pile(&mut self, selected_pile: Suit) -> Option<Card> {
        let pile = &mut self.piles[selected_pile as usize];
        pile.0.pop()
    }
}

impl Stock {
    pub fn new() -> Self {
        let mut cards: Vec<Card> = Value::ALL.iter()
            .flat_map(|&val| Suit::ALL.iter().map(move |&suit| Card::new(val, suit)))
            .collect();

        cards.shuffle(&mut rng());

        Stock { cards }
    }

    pub fn take(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn take_n(&mut self, n: usize) -> Vec<Card> {
        let take_amt = std::cmp::min(n, self.cards.len());
        let split_idx = self.cards.len() - take_amt;

        self.cards.split_off(split_idx)
    }
}

impl Klondike {
    pub fn new() -> Self {
        let mut stock = Stock::new();
        let tableau = Tableau::new(&mut stock);
        let foundation = Foundation::new();

        Self { tableau, foundation, stock, cards_in_play: (Vec::new(), CardTarget::default()) }
    }

    pub fn primary_action_at(&mut self, target: CardTarget) -> Result<(), GameError> {
        match target.location {
            CardLocation::Tableau => {
                if self.cards_in_play.0.is_empty() {
                    self.get_cards_in_play(target)?;
                } else {
                    self.place_cards_in_play(target)?;
                }
            }

            CardLocation::Foundation => {
                if self.cards_in_play.0.is_empty() {
                    if let Some(card) = self.foundation.take_card_from_pile(target.col_idx.into()) {
                        self.cards_in_play.0 = vec![card];
                        self.cards_in_play.1 = target;
                    }
                } else if self.cards_in_play.0.len() == 1 {
                    if let Some(card) = self.cards_in_play.0.pop() {
                        match self.foundation.place_card_on_pile(
                            card,
                            target.col_idx.into()
                        ) {
                            Ok(()) => {
                                if let Some(original_col) = self.tableau.cols.get_mut(self.cards_in_play.1.col_idx) {
                                    if let Some(target_idx) = self.cards_in_play.1.card_idx.checked_sub(1) {
                                        if let Some(card) = original_col.get_mut(target_idx) {
                                            card.face_up = true; 
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                self.force_restore_cards(vec![card], self.cards_in_play.1);
                            }
                        }
                    }
                }
            }

            CardLocation::Stock => {}
        }

        Ok(())
    }

    pub fn get_cards_in_play(&mut self, target: CardTarget) -> Result<(), GameError> {
        self.cards_in_play = (self.tableau.get_valid_cards_from(target).unwrap_or_default(), target);
        Ok(())
    }

    pub fn place_cards_in_play(&mut self, target: CardTarget) -> Result<(), GameError> {
        // TODO: Add error interaction
        match self.tableau.place_cards_in_col(&self.cards_in_play.0, target.col_idx) {
            Ok(()) => {
                // flip face down card at original col
                if let Some(original_col) = self.tableau.cols.get_mut(self.cards_in_play.1.col_idx) {
                    if let Some(target_idx) = self.cards_in_play.1.card_idx.checked_sub(1) {
                        if let Some(card) = original_col.get_mut(target_idx) {
                            card.face_up = true; 
                        }
                    }
                }
            },
            Err(_) => {
                // restore cards to original col
                self.force_restore_cards(self.cards_in_play.0.clone(), self.cards_in_play.1);
            }
        }
        self.cards_in_play.0.clear();
        Ok(())
    }

    fn force_restore_cards(&mut self, cards: Vec<Card>, target: CardTarget) {
        match target.location {
            CardLocation::Tableau => {
                if let Some(col) = self.tableau.cols.get_mut(target.col_idx) {
                    col.extend(&cards);
                }
            }
            CardLocation::Foundation => {
                if let Some(col) = self.foundation.piles.get_mut(target.col_idx) {
                    col.0.extend(&cards);
                }
            }
            CardLocation::Stock => {
                self.stock.cards.extend(&cards);
            }
        }
    }
}
