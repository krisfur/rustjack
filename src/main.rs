use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, size},
};
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

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

    // Helper to pad a line properly inside the box using Unicode width
    fn pad_line(&self, content: &str, total_width: usize) -> String {
        let display_width = UnicodeWidthStr::width(content);
        let padding = total_width.saturating_sub(display_width);
        format!("{}{}", content, " ".repeat(padding))
    }

    fn render(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // Clear screen and move cursor to top-left
        queue!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        let (term_width, term_height) = size()?;

        // Main window dimensions
        let window_width = 60;
        let window_height = 20;
        let start_x = (term_width.saturating_sub(window_width)) / 2;
        let start_y = (term_height.saturating_sub(window_height)) / 2;

        // Draw the main window
        self.draw_main_window(&mut stdout, start_x, start_y, window_width)?;

        // Draw popup if there's a result
        if !self.round_result.is_empty() {
            self.draw_popup(&mut stdout)?;
        }

        stdout.flush()?;
        Ok(())
    }

    fn draw_main_window(&self, stdout: &mut io::Stdout, start_x: u16, start_y: u16, width: u16) -> io::Result<()> {
        let inner_width = (width - 2) as usize; // Width inside the box borders

        // Draw top border with title
        queue!(stdout, cursor::MoveTo(start_x, start_y))?;
        write!(stdout, "┌{}┐\r", "─".repeat(inner_width))?;

        // Title
        let title = " ♠ BLACKJACK ♥ ";
        let title_x = start_x + (width - UnicodeWidthStr::width(title) as u16) / 2;
        queue!(stdout, cursor::MoveTo(title_x, start_y))?;
        write!(stdout, "{}\r", title)?;

        let mut line = start_y + 1;

        // Empty line
        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "│{}│\r", " ".repeat(inner_width))?;
        line += 1;

        // Dealer section header
        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "├{}┤\r", "─".repeat(inner_width))?;
        line += 1;

        queue!(stdout, cursor::MoveTo(start_x, line))?;
        let dealer_label = "  DEALER";
        write!(stdout, "│{}│\r", self.pad_line(dealer_label, inner_width))?;
        line += 1;

        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "├{}┤\r", "─".repeat(inner_width))?;
        line += 1;

        // Dealer's cards
        match self.state {
            GameState::PlayerTurn => {
                let cards = self.dealer_hand.display_str();
                let visible = cards.split_once(' ').map(|(_, rest)| rest).unwrap_or("");
                let display = format!("  Cards: [??] {}", visible);

                queue!(stdout, cursor::MoveTo(start_x, line))?;
                write!(stdout, "│{}│\r", self.pad_line(&display, inner_width))?;
                line += 1;

                let value_display = "  Value: ???";
                queue!(stdout, cursor::MoveTo(start_x, line))?;
                write!(stdout, "│{}│\r", self.pad_line(value_display, inner_width))?;
                line += 1;
            }
            _ => {
                let display = format!("  Cards: {}", self.dealer_hand.display_str());

                queue!(stdout, cursor::MoveTo(start_x, line))?;
                write!(stdout, "│{}│\r", self.pad_line(&display, inner_width))?;
                line += 1;

                let value_display = format!("  Value: {}", self.dealer_hand.value());
                queue!(stdout, cursor::MoveTo(start_x, line))?;
                write!(stdout, "│{}│\r", self.pad_line(&value_display, inner_width))?;
                line += 1;
            }
        }

        // Empty line
        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "│{}│\r", " ".repeat(inner_width))?;
        line += 1;

        // Player section header
        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "├{}┤\r", "─".repeat(inner_width))?;
        line += 1;

        queue!(stdout, cursor::MoveTo(start_x, line))?;
        let player_label = "  PLAYER";
        write!(stdout, "│{}│\r", self.pad_line(player_label, inner_width))?;
        line += 1;

        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "├{}┤\r", "─".repeat(inner_width))?;
        line += 1;

        // Player's cards
        let player_display = format!("  Cards: {}", self.player_hand.display_str());
        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "│{}│\r", self.pad_line(&player_display, inner_width))?;
        line += 1;

        let player_value = format!("  Value: {}", self.player_hand.value());
        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "│{}│\r", self.pad_line(&player_value, inner_width))?;
        line += 1;

        // Empty line
        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "│{}│\r", " ".repeat(inner_width))?;
        line += 1;

        // Controls section
        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "├{}┤\r", "─".repeat(inner_width))?;
        line += 1;

        let controls = match self.state {
            GameState::PlayerTurn => "  [H] Hit  │  [S] Stand  │  [Q] Quit",
            GameState::RoundEnd => "  [N] New Round  │  [Q] Quit",
            _ => "  [Q] Quit",
        };

        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "│{}│\r", self.pad_line(controls, inner_width))?;
        line += 1;

        // Empty line
        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "│{}│\r", " ".repeat(inner_width))?;
        line += 1;

        // Bottom border
        queue!(stdout, cursor::MoveTo(start_x, line))?;
        write!(stdout, "└{}┘\r", "─".repeat(inner_width))?;

        Ok(())
    }

    fn draw_popup(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        let (term_width, term_height) = size()?;

        // Popup dimensions
        let popup_width = 50;
        let popup_height = 7;
        let start_x = (term_width.saturating_sub(popup_width)) / 2;
        let start_y = (term_height.saturating_sub(popup_height)) / 2;

        // Draw shadow (optional, for depth effect)
        for i in 0..popup_height {
            queue!(stdout, cursor::MoveTo(start_x + 1, start_y + i + 1))?;
            write!(stdout, "{}", " ".repeat(popup_width as usize))?;
        }

        // Draw popup box
        queue!(stdout, cursor::MoveTo(start_x, start_y))?;
        write!(stdout, "┌{}┐\r", "─".repeat(popup_width as usize - 2))?;

        for i in 1..popup_height - 1 {
            queue!(stdout, cursor::MoveTo(start_x, start_y + i))?;
            write!(stdout, "│{}│\r", " ".repeat(popup_width as usize - 2))?;
        }

        queue!(stdout, cursor::MoveTo(start_x, start_y + popup_height - 1))?;
        write!(stdout, "└{}┘\r", "─".repeat(popup_width as usize - 2))?;

        // Draw title
        queue!(stdout, cursor::MoveTo(start_x + 2, start_y + 1))?;
        write!(stdout, "ROUND RESULT\r")?;

        // Draw separator
        queue!(stdout, cursor::MoveTo(start_x, start_y + 2))?;
        write!(stdout, "├{}┤\r", "─".repeat(popup_width as usize - 2))?;

        // Draw the result message (centered)
        let result_width = UnicodeWidthStr::width(self.round_result.as_str());
        let result_x = start_x + ((popup_width as usize - result_width) / 2) as u16;
        queue!(stdout, cursor::MoveTo(result_x, start_y + 3))?;
        write!(stdout, "{}\r", self.round_result)?;

        // Draw prompt
        let prompt = "Press [N] for new round or [Q] to quit";
        let prompt_width = UnicodeWidthStr::width(prompt);
        let prompt_x = start_x + ((popup_width as usize - prompt_width) / 2) as u16;
        queue!(stdout, cursor::MoveTo(prompt_x, start_y + 5))?;
        write!(stdout, "{}\r", prompt)?;

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