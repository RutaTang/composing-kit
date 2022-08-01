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
    style::{self, Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

//todo: show menuinfo when a certain menu item is selcted
fn ui<B: Backend>(
    f: &mut Frame<B>,
    menuselect_state: &mut MenuSelectState,
    board_state: &mut BoardState,
) {
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
    let mainboard = match board_state.current_board {
        Board::Main => mainboard.border_style(Style::default().fg(Color::Yellow)),
        _ => mainboard,
    };
    f.render_widget(mainboard, dashboard[0]);

    let menuinfoboard = Block::default().title("Menu Info").borders(Borders::ALL);
    let menuinfoboard = match board_state.current_board{
        Board::MenuInfo => menuinfoboard.border_style(Style::default().fg(Color::Yellow)),
        _ => menuinfoboard,
    };
    f.render_widget(menuinfoboard, menu[0]);

    let menuselectboard = Block::default().title("Menu Select").borders(Borders::ALL);
    let menuselectboard = match board_state.current_board {
        Board::MenuSelect => menuselectboard.border_style(Style::default().fg(Color::Yellow)),
        _ => menuselectboard,
    };
    let items: Vec<ListItem> = menuselect_state
        .items
        .iter()
        .map(|item| ListItem::new(item.to_string()))
        .collect();
    let list = List::new(items)
        .block(menuselectboard)
        .highlight_style(Style::default().fg(Color::Yellow));
    f.render_stateful_widget(list, menu[1], &mut menuselect_state.state);
}

#[derive(Debug)]
enum InputEvent<I> {
    Input(I),
    Tick,
}

#[derive(Debug)]
enum Board {
    Main,
    MenuInfo,
    MenuSelect,
}

#[derive(Debug)]
struct BoardState {
    current_board: Board,
}
impl BoardState {
    fn new() -> Self {
        BoardState {
            current_board: Board::MenuSelect,
        }
    }
    fn select_board(&mut self, board: Board) {
        self.current_board = board;
    }
}

#[derive(Debug)]
struct MenuSelectState {
    items: Vec<String>,
    state: ListState,
}
impl MenuSelectState {
    fn new() -> Self {
        Self {
            items: vec![],
            state: ListState::default(),
        }
    }
    fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.state.select(Some(0));
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => self.items.len() - 1,
        };
        self.state.select(Some(i));
    }
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
        if (event::poll(Duration::from_millis(100)).unwrap()) {
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
    let mut board_state = BoardState::new();
    let mut menuselect_state = MenuSelectState::new();
    menuselect_state.set_items(vec!["Circle of fifth".into()]);
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
                KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                } => {
                    menuselect_state.next();
                }
                KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                } => {
                    menuselect_state.previous();
                }
                KeyEvent {
                    code: KeyCode::Char('u'),
                    modifiers: KeyModifiers::NONE,
                } => {
                        board_state.select_board(Board::Main);
                    }
                KeyEvent {
                    code: KeyCode::Char('i'),
                    modifiers: KeyModifiers::NONE,
                } => {
                        board_state.select_board(Board::MenuInfo);
                    }
                KeyEvent {
                    code: KeyCode::Char('o'),
                    modifiers: KeyModifiers::NONE,
                } => {
                        board_state.select_board(Board::MenuSelect);
                    }
                _ => {}
            },
            InputEvent::Tick => {}
        }
        terminal.draw(|f| {
            ui(f, &mut menuselect_state, &mut board_state);
        })?;
    }

    Ok(())
}
