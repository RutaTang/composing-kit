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
    text::Text,
    widgets::{
        canvas::{Canvas, Line, Map, MapResolution, Points, Rectangle},
        Block, Borders, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame, Terminal,
};

//todo: draw circle of fifth graph
fn ui<B: Backend>(f: &mut Frame<B>, menu_state: &mut MenuState, board_state: &mut BoardState) {
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
    let mainboard = Canvas::default()
        .block(mainboard)
        .x_bounds([-200.0, 200.0])
        .y_bounds([-200.0, 200.0])
        .paint(|ctx| {
            //todo: draw key note
            ctx.draw(&Points {
                coords: &[(0.0, 0.0)],
                color: Color::White,
            });
            const RADIUS: u16 = 150;
            for degree in 0..=360/20 {
                let degree = (20*degree) as f64;
                let degree = degree / 360_f64 * std::f64::consts::PI;
                let y = degree.sin() * (RADIUS as f64);
                let x = degree.cos() * (RADIUS as f64);
                ctx.draw(&Points {
                    coords: &[(x, y)],
                    color: Color::White,
                })
            }
            for degree in 0..=360/20 {
                let degree = (20*degree) as f64;
                let degree = degree / 360_f64 * std::f64::consts::PI;
                let y = degree.sin() * (RADIUS as f64);
                let x = degree.cos() * (RADIUS as f64);
                ctx.draw(&Points {
                    coords: &[(-x, -y)],
                    color: Color::White,
                })
            }
        });
    f.render_widget(mainboard, dashboard[0]);

    let menuinfoboard = Block::default().title("Menu Info").borders(Borders::ALL);
    let menuinfoboard = match board_state.current_board {
        Board::MenuInfo => menuinfoboard.border_style(Style::default().fg(Color::Yellow)),
        _ => menuinfoboard,
    };
    let selected_idx = menu_state.state.selected().unwrap();
    let menuinfo_text = Text::raw(menu_state.infos.get(selected_idx).unwrap());
    let menuinfo_text = Paragraph::new(menuinfo_text)
        .block(menuinfoboard)
        .scroll((menu_state.info_scrolls[selected_idx], 0))
        .wrap(Wrap { trim: true });
    f.render_widget(menuinfo_text, menu[0]);

    let menuselectboard = Block::default().title("Menu Select").borders(Borders::ALL);
    let menuselectboard = match board_state.current_board {
        Board::MenuSelect => menuselectboard.border_style(Style::default().fg(Color::Yellow)),
        _ => menuselectboard,
    };
    let items: Vec<ListItem> = menu_state
        .items
        .iter()
        .map(|item| ListItem::new(item.to_string()))
        .collect();
    let list = List::new(items)
        .block(menuselectboard)
        .highlight_style(Style::default().fg(Color::Yellow));
    f.render_stateful_widget(list, menu[1], &mut menu_state.state);
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
struct MenuState {
    items: Vec<String>,
    infos: Vec<String>,
    info_scrolls: Vec<u16>,
    state: ListState,
}
impl MenuState {
    fn new() -> Self {
        Self {
            items: vec![],
            infos: vec![],
            state: ListState::default(),
            info_scrolls: vec![],
        }
    }
    fn set_items(&mut self, items: Vec<String>, infos: Vec<String>) {
        assert!(items.len() == infos.len(), "shold be same length");
        self.items = items;
        self.infos = infos;
        self.info_scrolls = vec![0; self.infos.len()];
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
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn scroll_up(&mut self, idx: usize) {
        self.info_scrolls[idx] = self.info_scrolls[idx].saturating_sub(1);
    }
    fn scroll_down(&mut self, idx: usize) {
        self.info_scrolls[idx] = self.info_scrolls[idx].saturating_add(1);
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
    let mut menu_state = MenuState::new();
    menu_state.set_items(
        vec!["Circle of fifth".into(), "Harmonic".into()],
        vec![
            "This is a info for COF Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet. Nisi anim cupidatat excepteur officia. Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident. Nostrud officia pariatur ut officia. Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate. Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod. Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim. Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis.Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet. Nisi anim cupidatat excepteur officia. Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident. Nostrud officia pariatur ut officia. Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate. Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod. Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim. Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis. Ruta!".into(),
            "Oh yes, a test text".into(),
        ],
    );
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
                    code: KeyCode::Down | KeyCode::Char('j'),
                    modifiers: KeyModifiers::NONE,
                } => match board_state.current_board {
                    Board::MenuSelect => menu_state.next(),
                    Board::MenuInfo => menu_state.scroll_down(menu_state.state.selected().unwrap()),
                    _ => {}
                },
                KeyEvent {
                    code: KeyCode::Up | KeyCode::Char('k'),
                    modifiers: KeyModifiers::NONE,
                } => match board_state.current_board {
                    Board::MenuSelect => menu_state.previous(),
                    Board::MenuInfo => menu_state.scroll_up(menu_state.state.selected().unwrap()),
                    _ => {}
                },
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
            ui(f, &mut menu_state, &mut board_state);
        })?;
    }

    Ok(())
}
