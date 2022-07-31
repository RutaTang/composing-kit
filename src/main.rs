use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, Stdout},
    sync::mpsc,
    thread,
    time::Duration,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, self},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

fn ui<B: Backend>(f: &mut Frame<B>, menu_state: &mut MenuState) {
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
    let items: Vec<ListItem> = menu_state
        .items
        .iter()
        .map(|item| ListItem::new(item.to_string()))
        .collect();
    let list = List::new(items).block(menuselectboard).highlight_style(Style::default().fg(Color::Yellow));
    f.render_stateful_widget(list, menu[1], &mut menu_state.state);
}

#[derive(Debug)]
enum InputEvent<I> {
    Input(I),
    Tick,
}

#[derive(Debug)]
struct MenuState {
    items: Vec<String>,
    state: ListState,
}
impl MenuState {
    fn new(items: Vec<String>) -> Self {
        Self {
            items,
            state: ListState::default(),
        }
    }
    //todo: implement next/previous function to navigate
}


fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel();

    //Use input event thread to listen key event and send to ui thread
    thread::spawn(move || loop {
        if (event::poll(Duration::from_millis(200)).unwrap()) {
            let ev = event::read().unwrap();
            match ev {
                Event::Key(key) => {
                    tx.send(InputEvent::Input(key)).unwrap();
                }
                _ => {}
            }
        } else {
            tx.send(InputEvent::Tick).unwrap();
        }
    });

    // ui thread
    let mut menu_state = MenuState::new(vec!["Circle of fifths".to_string(),"Second Staff".to_string()]);
    loop {
        let input_event = rx.recv().unwrap();
        match input_event {
            InputEvent::Input(key) => match key {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                } => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                //Todo: implement keyup and down
                KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                } => {
                    menu_state.state.select(Some(1));
                }
                _ => {}
            },
            InputEvent::Tick => {}
        }
        terminal.draw(|f| {
            ui(f, &mut menu_state);
        })?;
    }

    Ok(())
}
