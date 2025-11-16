use std::io;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use ratatui::prelude::*;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Block, Borders, List, ListItem};

use locust::core::context::{Locust, LocustConfig};
use locust::plugins::nav::NavPlugin;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut locust: Locust<_> = Locust::new(LocustConfig::default());
    locust.register_plugin(NavPlugin::new());

    let items: Vec<ListItem> = (0..20)
        .map(|i| ListItem::new(format!("Item {}", i)))
        .collect();

    'outer: loop {
        locust.begin_frame();
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("Locust demo").borders(Borders::ALL);
            f.render_widget(block, size);

            let inner = Rect {
                x: size.x + 1,
                y: size.y + 1,
                width: size.width - 2,
                height: size.height - 2,
            };

            let list = List::new(items.clone());
            f.render_widget(list, inner);

            // Finally, let Locust render any overlays.
            locust.render_overlay(f);
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            let ev = event::read()?;
            let outcome = locust.on_event(&ev);
            if !outcome.consumed {
                if let Event::Key(key) = ev {
                    if key.code == KeyCode::Char('q') {
                        break 'outer;
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::event::DisableMouseCapture,
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}
