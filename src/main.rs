use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, thread, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame, Terminal,
};

fn ui<B: Backend>(f: &mut Frame<B>) {
    let dashboard = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(f.size());
    let menu = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(dashboard[1]);

    let mainboard = Block::default().title("Main Board").borders(Borders::ALL);
    f.render_widget(mainboard, dashboard[0]);

    let menuinfoboard = Block::default().title("Menu Info").borders(Borders::ALL);
    f.render_widget(menuinfoboard, menu[0]);

    let menuselectboard = Block::default().title("Menu Select").borders(Borders::ALL);
    f.render_widget(menuselectboard, menu[1]);
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        ui(f);
    })?;

    loop {
        match event::read()? {
            Event::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Char('q') => {
                    // restore terminal
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}
