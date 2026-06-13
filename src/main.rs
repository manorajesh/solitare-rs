mod card;
mod logic;
mod renderer;

use crate::{ logic::Klondike, renderer::* };

fn main() -> std::io::Result<()> {
    let mut term = Terminal::setup()?;
    let mut game = Klondike::new(|stats| {
        eprintln!("Game Over! Statistics: {:?}", stats);
    });
    let status = run_game(&mut term, &mut game);
    term.destroy()?;

    status
}

fn run_game(term: &mut Terminal, game: &mut Klondike) -> std::io::Result<()> {
    loop {
        term.clear()?;
        game.draw(term, 0, 0, false)?;

        match term.read()? {
            GameEvent::Quit => {
                break;
            }
            GameEvent::MouseMove { col, row } => {
                term.set_hovered_cards(game.get_card_at(col, row));
            }
            GameEvent::MouseDown { col, row } => {
                if let Some(target) = game.get_card_at(col, row) {
                    game.primary_action_at(target)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}