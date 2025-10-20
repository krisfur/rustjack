use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
};
use std::io::{self, Write};

mod game;
use game::{Deck, Hand};

enum GameState {
    PlayerTurn,
    DealerTurn,
    RoundEnd,
    GameOver,
}

struct GameUI {
    state: GameState,
    deck: Deck,
    player_hand: Hand,
    dealer_hand: Hand,
    round_result: String,
}

impl GameUI {
    fn new() -> Self {
        let mut deck = Deck::new();
        deck.shuffle();

        let mut player_hand = Hand::new();
        let mut dealer_hand = Hand::new();

        // Initial deal: 2 cards each, alternating player/dealer
        for _ in 0..2 {
            player_hand.add_card(deck.deal().unwrap());
            dealer_hand.add_card(deck.deal().unwrap());
        }

        Self {
            state: GameState::PlayerTurn,
            deck,
            player_hand,
            dealer_hand,
            round_result: String::new(),
        }
    }

    fn reset_round(&mut self) {
        self.deck = Deck::new();
        self.deck.shuffle();
        self.player_hand = Hand::new();
        self.dealer_hand = Hand::new();

        // Initial deal: 2 cards each, alternating player/dealer
        for _ in 0..2 {
            self.player_hand.add_card(self.deck.deal().unwrap());
            self.dealer_hand.add_card(self.deck.deal().unwrap());
        }

        self.state = GameState::PlayerTurn;
        self.round_result = String::new();
    }

    fn render(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Clear screen and move cursor to top-left
        queue!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        let mut line = 0;

        queue!(stdout, cursor::MoveTo(0, line))?;
        write!(stdout, "===============================================\r")?;
        line += 1;

        queue!(stdout, cursor::MoveTo(0, line))?;
        write!(stdout, "        BLACKJACK TUI\r")?;
        line += 1;

        queue!(stdout, cursor::MoveTo(0, line))?;
        write!(stdout, "===============================================\r")?;
        line += 2;

        // Dealer's hand
        queue!(stdout, cursor::MoveTo(0, line))?;
        write!(stdout, "--- Dealer's Hand ---\r")?;
        line += 1;

        match self.state {
            GameState::PlayerTurn => {
                let cards = self.dealer_hand.display_str();
                let visible = cards.split_once(' ').map(|(_, rest)| rest).unwrap_or("");

                queue!(stdout, cursor::MoveTo(0, line))?;
                write!(stdout, "Cards: [??] {}\r", visible)?;
                line += 1;

                queue!(stdout, cursor::MoveTo(0, line))?;
                write!(stdout, "Value: ???\r")?;
                line += 1;
            }
            _ => {
                queue!(stdout, cursor::MoveTo(0, line))?;
                write!(stdout, "Cards: {}\r", self.dealer_hand.display_str())?;
                line += 1;

                queue!(stdout, cursor::MoveTo(0, line))?;
                write!(stdout, "Value: {}\r", self.dealer_hand.value())?;
                line += 1;
            }
        }
        line += 1;

        // Player's hand
        queue!(stdout, cursor::MoveTo(0, line))?;
        write!(stdout, "--- Your Hand ---\r")?;
        line += 1;

        queue!(stdout, cursor::MoveTo(0, line))?;
        write!(stdout, "Cards: {}\r", self.player_hand.display_str())?;
        line += 1;

        queue!(stdout, cursor::MoveTo(0, line))?;
        write!(stdout, "Value: {}\r", self.player_hand.value())?;
        line += 2;

        // Result message
        if !self.round_result.is_empty() {
            queue!(stdout, cursor::MoveTo(0, line))?;
            write!(stdout, "--- Result ---\r")?;
            line += 1;

            queue!(stdout, cursor::MoveTo(0, line))?;
            write!(stdout, "{}\r", self.round_result)?;
            line += 2;
        }

        // Controls
        queue!(stdout, cursor::MoveTo(0, line))?;
        write!(stdout, "--- Controls ---\r")?;
        line += 1;

        queue!(stdout, cursor::MoveTo(0, line))?;
        match self.state {
            GameState::PlayerTurn => {
                write!(stdout, "[H] Hit  |  [S] Stand  |  [Q] Quit\r")?;
            }
            GameState::RoundEnd => {
                write!(stdout, "[N] New Round  |  [Q] Quit\r")?;
            }
            _ => {
                write!(stdout, "[Q] Quit\r")?;
            }
        }
        line += 1;

        queue!(stdout, cursor::MoveTo(0, line))?;
        write!(stdout, "===============================================\r")?;

        stdout.flush()?;
        Ok(())
    }

    fn handle_player_turn(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('h') | KeyCode::Char('H') => {
                let new_card = self.deck.deal().unwrap();
                self.player_hand.add_card(new_card);

                if self.player_hand.value() > 21 {
                    self.round_result = String::from("BUST! You lose this round.");
                    self.state = GameState::RoundEnd;
                } else if self.player_hand.value() == 21 {
                    self.state = GameState::DealerTurn;
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.state = GameState::DealerTurn;
            }
            _ => {}
        }
    }

    fn resolve_dealer_turn(&mut self) {
        // Dealer plays
        while self.dealer_hand.value() < 17 {
            let new_card = self.deck.deal().unwrap();
            self.dealer_hand.add_card(new_card);
        }

        // Determine winner
        let player_score = self.player_hand.value();
        let dealer_score = self.dealer_hand.value();

        if dealer_score > 21 {
            self.round_result = String::from("Dealer busts! You win!");
        } else if player_score > dealer_score {
            self.round_result = format!("You win! ({} vs {})", player_score, dealer_score);
        } else if player_score < dealer_score {
            self.round_result = format!("You lose. ({} vs {})", player_score, dealer_score);
        } else {
            self.round_result = format!("Push! It's a tie at {}", player_score);
        }
        
        self.state = GameState::RoundEnd;
    }

    fn handle_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                self.state = GameState::GameOver;
                return false;
            }
            _ => {}
        }

        match self.state {
            GameState::PlayerTurn => {
                self.handle_player_turn(key);
                if matches!(self.state, GameState::DealerTurn) {
                    self.resolve_dealer_turn();
                }
            }
            GameState::RoundEnd => {
                if let KeyCode::Char('n') | KeyCode::Char('N') = key {
                    self.reset_round();
                }
            }
            _ => {}
        }

        true
    }

    fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        self.render()?;

        loop {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                if !self.handle_input(code) {
                    break;
                }
                self.render()?;
            }
        }

        disable_raw_mode()?;
        execute!(stdout, LeaveAlternateScreen)?;
        println!("\nThanks for playing!");

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut game = GameUI::new();
    game.run()
}