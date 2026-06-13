use crate::{ card::{ self, Card, Suit }, logic::{Foundation, Klondike, Stock, Tableau} };

use crossterm::{
    cursor,
    event::{
        self,
        DisableMouseCapture,
        EnableMouseCapture,
        Event,
        KeyCode,
        KeyEventKind,
        MouseButton,
        MouseEventKind,
    },
    execute,
    style::{ Color, ResetColor, SetForegroundColor },
    terminal::{ self, EnterAlternateScreen, LeaveAlternateScreen },
};
use std::io::{ self, stdout, Stdout, Write };

pub enum GameEvent {
    Quit,
    MouseDown {
        col: u16,
        row: u16,
    },
    MouseDrag {
        col: u16,
        row: u16,
    },
    MouseMove {
        col: u16,
        row: u16,
    },
    Resize {
        width: u16,
        height: u16,
    },
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardLocation {
    #[default]
    Tableau,
    Foundation,
    Stock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardTarget {
    pub location: CardLocation,
    pub col_idx: usize,
    pub card_idx: usize,
}

pub struct Terminal {
    stdout: Stdout,
    hovered_cards: Option<CardTarget>,
    mouse_pos: (u16, u16),
}

pub trait Draw {
    fn draw(&self, term: &mut Terminal, col: u16, row: u16, is_active: bool) -> io::Result<()>;
    fn draw_placeholder(&self, _term: &mut Terminal, _col: u16, _row: u16, _is_active: bool, _show_suit: bool) -> io::Result<()> {Ok(())}
}

impl Suit {
    fn get_color(&self) -> Color {
        match self {
            Suit::Hearts | Suit::Diamonds => Color::Red,
            Suit::Clubs | Suit::Spades => Color::Reset,
        }
    }

    fn get_dim_color(&self) -> Color {
        match self {
            Suit::Hearts | Suit::Diamonds => Color::DarkRed,
            Suit::Clubs | Suit::Spades => Color::DarkGrey,
        }
    }
}

impl Draw for Card {
    fn draw(&self, term: &mut Terminal, col: u16, row: u16, is_active: bool) -> io::Result<()> {
        if is_active {
            execute!(term.stdout, SetForegroundColor(Color::Yellow))?;
        }

        if !self.face_up {
            term.move_to(col, row)?;
            write!(term.stdout, "╭─────╮")?;
            term.move_to(col, row + 1)?;
            write!(term.stdout, "│\\\\\\\\\\│")?;
            term.move_to(col, row + 2)?;
            write!(term.stdout, "│\\\\\\\\\\│")?;
            term.move_to(col, row + 3)?;
            write!(term.stdout, "│\\\\\\\\\\│")?;
            term.move_to(col, row + 4)?;
            write!(term.stdout, "╰─────╯")?;

            if is_active {
                execute!(term.stdout, ResetColor)?;
            }
            return Ok(());
        }

        let reset_color = if is_active {
            Color::Yellow
        } else {
            Color::Reset
        };

        let value = self.value.to_string();
        let suit = self.suit.to_string();
        let text_color = self.suit.get_color();

        term.write_at(col, row, "╭─────╮")?;

        term.move_to(col, row + 1)?;
        write!(term.stdout, "│")?;
        execute!(term.stdout, SetForegroundColor(text_color))?;
        write!(term.stdout, "{:<5}", value)?;
        execute!(term.stdout, SetForegroundColor(reset_color))?;
        write!(term.stdout, "│")?;

        term.move_to(col, row + 2)?;
        write!(term.stdout, "│  ")?;
        execute!(term.stdout, SetForegroundColor(text_color))?;
        write!(term.stdout, "{}", suit)?;
        execute!(term.stdout, SetForegroundColor(reset_color))?;
        write!(term.stdout, "  │")?;

        term.move_to(col, row + 3)?;
        write!(term.stdout, "│")?;
        execute!(term.stdout, SetForegroundColor(text_color))?;
        write!(term.stdout, "{:>5}", value)?;
        execute!(term.stdout, SetForegroundColor(reset_color))?;
        write!(term.stdout, "│")?;

        // Bottom line: Draw standard border
        term.write_at(col, row + 4, "╰─────╯")?;

        if is_active {
            execute!(term.stdout, ResetColor)?;
        }
        Ok(())
    }

    fn draw_placeholder(&self, term: &mut Terminal, col: u16, row: u16, is_active: bool, show_suit: bool) -> io::Result<()> {
        let suit = self.suit.to_string();
        let suit_color = self.suit.get_dim_color();
        let card_color = if is_active {
            Color::Yellow
        } else {
            Color::DarkGrey
        };

        execute!(term.stdout, SetForegroundColor(card_color))?;

        term.write_at(col, row, "╭─────╮")?;
        term.write_at(col, row + 1, "│     │")?;

        term.move_to(col, row + 2)?;
        write!(term.stdout, "│  ")?;
        execute!(term.stdout, SetForegroundColor(suit_color))?;
        write!(term.stdout, "{}", if show_suit {suit} else {" ".to_string()})?;
        execute!(term.stdout, SetForegroundColor(card_color))?;
        write!(term.stdout, "  │")?;

        term.write_at(col, row + 3, "│     │")?;
        term.write_at(col, row + 4, "╰─────╯")?;

        execute!(term.stdout, ResetColor)?;
        Ok(())
    }
}

impl Draw for Tableau {
    fn draw(&self, term: &mut Terminal, col: u16, row: u16, _is_active: bool) -> io::Result<()> {
        for (col_idx, col_cards) in self.cols.iter().enumerate() {
            let x = (col_idx as u16) * 8 + col;
            let mut y = row;

            let mut is_active = false;
            if col_cards.is_empty() {
                if let Some(target) = term.hovered_cards {
                    if target.location == CardLocation::Tableau && target.col_idx == col_idx {
                        let card = Card::new(card::Value::Ace, col_idx.into(), false);
                        card.draw_placeholder(term, x, y, true, false)?;
                        continue;
                    }
                }
            }
            for (card_idx, card) in col_cards.iter().enumerate() {
                if let Some(target) = term.hovered_cards {
                    if !is_active && 
                       target.location == CardLocation::Tableau && 
                       target.col_idx == col_idx && 
                       target.card_idx == card_idx && 
                       card.face_up 
                    {
                        is_active = true;
                    }
                } else {
                    is_active = false;
                }

                card.draw(term, x, y, is_active)?;
                y += 2;
            }
        }

        Ok(())
    }
}

impl Draw for Foundation {
    fn draw(&self, term: &mut Terminal, col: u16, row: u16, is_active: bool) -> io::Result<()> {
        for (col_idx, col_cards) in self.piles.iter().enumerate() {
            let x = (col_idx as u16) * 8 + col;
            let y = row;

            let is_active = if let Some(target) = term.hovered_cards {
                target.location == CardLocation::Foundation && target.col_idx == col_idx
            } else {
                false
            };

            if let Some(card) = col_cards.0.last() {
                card.draw(term, x, y, is_active)?;
            } else {
                // TODO: Need to allocate one more?
                let card = Card::new(card::Value::Ace, col_idx.into(), false);
                card.draw_placeholder(term, x, y, is_active, true)?;
            }
        }

        Ok(())
    }
}

impl Draw for Stock {
    fn draw(&self, term: &mut Terminal, col: u16, row: u16, is_active: bool) -> io::Result<()> {
        // draw stack (face down or repeat icon shown if all cards were drawn)
        let card = Card::default();
        let is_active = if let Some(target) = term.hovered_cards {
            target.location == CardLocation::Stock && target.col_idx == 0
        } else {
            false
        };
        card.draw(term, col, row, is_active)?;

        // draw visible card if there is
        let is_active = if let Some(target) = term.hovered_cards {
            target.location == CardLocation::Stock && target.col_idx == 1
        } else {
            false
        };
        if let Some(visible_card) = self.peek_visible() {
            visible_card.draw(term, col + 8, row, is_active)?;
        }
        Ok(())
    }
}

impl Draw for Klondike {
    fn draw(&self, term: &mut Terminal, col: u16, row: u16, is_active: bool) -> io::Result<()> {
        self.tableau.draw(term, col, row + 5, is_active)?;
        self.foundation.draw(term, col + 24, row, is_active)?;
        self.stock.draw(term, col, row, is_active)?;

        let mut y = term.mouse_pos.1;
        for card in &self.cards_in_play.0 {
            card.draw(term, term.mouse_pos.0, y, true)?;
            y += 2;
        }

        term.flush()?;
        Ok(())
    }
}

impl Terminal {
    pub fn setup() -> Result<Self, io::Error> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, cursor::Hide, EnableMouseCapture)?;
        Ok(Self { stdout, hovered_cards: None, mouse_pos: (0, 0) })
    }

    pub fn destroy(&mut self) -> io::Result<()> {
        execute!(self.stdout, LeaveAlternateScreen, cursor::Show, DisableMouseCapture)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn move_to(&mut self, column: u16, row: u16) -> io::Result<()> {
        execute!(self.stdout, cursor::MoveTo(column, row))?;
        Ok(())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        execute!(self.stdout, terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()?;
        Ok(())
    }

    pub fn read(&mut self) -> io::Result<GameEvent> {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    Ok(GameEvent::Quit)
                } else {
                    Ok(GameEvent::Ignore)
                }
            }

            Event::Mouse(mouse) =>
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        Ok(GameEvent::MouseDown { col: mouse.column, row: mouse.row })
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        Ok(GameEvent::MouseDrag { col: mouse.column, row: mouse.row })
                    }
                    MouseEventKind::Moved => {
                        self.mouse_pos = (mouse.column, mouse.row);
                        Ok(GameEvent::MouseMove { col: mouse.column, row: mouse.row })
                    }
                    _ => Ok(GameEvent::Ignore),
                }

            Event::Resize(w, h) => Ok(GameEvent::Resize { width: w, height: h }),
            _ => Ok(GameEvent::Ignore),
        }
    }

    pub fn write_at(&mut self, col: u16, row: u16, text: &str) -> io::Result<()> {
        self.move_to(col, row)?;
        write!(self.stdout, "{}", text)?;
        Ok(())
    }

    pub fn set_hovered_cards(&mut self, target: Option<CardTarget>) {
        self.hovered_cards = target;
    }
}

impl Klondike {
    pub fn get_card_at(&self, mouse_col: u16, mouse_row: u16) -> Option<CardTarget> {
        // stock
        for i in 0..2 {
            let start_x = (i as u16) * 8;
            let end_x = start_x + 7;

            let card_height = 5;

            let start_y = 0;
            let end_y = start_y + card_height;

            if mouse_col >= start_x && mouse_col < end_x && mouse_row >= start_y && mouse_row < end_y {
                return Some(CardTarget { col_idx: i, card_idx: 0, location: CardLocation::Stock });
            }
        }

        // foundation
        for col_idx in 0..self.foundation.piles.len() {
            let start_x = (col_idx as u16) * 8 + 24;
            let end_x = start_x + 7;

            if mouse_col < start_x || mouse_col >= end_x {
                continue;
            }

            let card_height = 5;

            let start_y = 0;
            let end_y = start_y + card_height;

            if mouse_row >= start_y && mouse_row < end_y {
                return Some(CardTarget { col_idx, card_idx: 0, location: CardLocation::Foundation });
            }
        }

        // tableau
        for (col_idx, col) in self.tableau.cols.iter().enumerate() {
            let start_x = (col_idx as u16) * 8;
            let end_x = start_x + 7;

            if mouse_col < start_x || mouse_col >= end_x {
                continue;
            }

            let mut current_y = 5;
            let total_cards = col.len();

            for (card_idx, _card) in col.iter().enumerate() {
                let is_last_card = card_idx == total_cards - 1;

                let card_height = if is_last_card {
                    5
                } else {
                    2
                };

                let start_y = current_y;
                let end_y = start_y + card_height;

                if mouse_row >= start_y && mouse_row < end_y {
                    return Some(CardTarget { col_idx, card_idx, location: CardLocation::Tableau });
                }

                current_y += 2;
            }

            // If column is empty, allow clicking on the empty space to select the column
            if col.is_empty() {
                let start_y = 5;
                let end_y = start_y + 5;

                if mouse_row >= start_y && mouse_row < end_y {
                    return Some(CardTarget { col_idx, card_idx: 0, location: CardLocation::Tableau });
                }
            }
        }

        None
    }
}