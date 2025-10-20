use rand::seq::SliceRandom;
use std::fmt;

// Represents the four suits of a card deck.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

// Represents the 13 ranks of a card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    /// Returns the primary blackjack value for a card rank.
    /// Ace is initially counted as 11.
    pub fn value(&self) -> u8 {
        match *self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
            Rank::Ace => 11,
        }
    }
}

// A single playing card with a suit and rank.
#[derive(Debug, Clone, Copy)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Card { suit, rank }
    }
}

// For displaying the card in a user-friendly way.
impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rank = match self.rank {
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ace => "A",
        };
        let suit = match self.suit {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        };
        write!(f, "{}{}", rank, suit)
    }
}

// Represents a deck of cards.
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    /// Creates a new, standard 52-card deck.
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);
        let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
        let ranks = [
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
            Rank::Ace,
        ];

        for &suit in &suits {
            for &rank in &ranks {
                cards.push(Card::new(suit, rank));
            }
        }
        Deck { cards }
    }

    /// Shuffles the deck randomly.
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }

    /// Deals one card from the top of the deck.
    pub fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}

// Represents a player's or dealer's hand.
pub struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        Hand { cards: Vec::new() }
    }

    /// Adds a card to the hand.
    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Calculates the total value of the hand.
    /// It correctly handles the flexible value of Aces (1 or 11).
    pub fn value(&self) -> u8 {
        let mut value = 0;
        let mut ace_count = 0;

        for card in &self.cards {
            value += card.rank.value();
            if card.rank == Rank::Ace {
                ace_count += 1;
            }
        }

        // Adjust for Aces if the total value is over 21
        while value > 21 && ace_count > 0 {
            value -= 10;
            ace_count -= 1;
        }

        value
    }

    /// Returns a string representation of all cards in the hand.
    pub fn display_str(&self) -> String {
        self.cards
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}
