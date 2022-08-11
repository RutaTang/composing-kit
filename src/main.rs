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
    text::{Span, Text},
    widgets::{
        canvas::{Canvas, Context, Label, Line, Map, MapResolution, Points, Rectangle},
        Block, Borders, List, ListItem, ListState, Paragraph, Widget, Wrap,
    },
    Frame, Terminal,
};

//return a customizable canvas with cof drawn
fn draw_circle_of_fifths<'a>() -> Canvas<'a, impl Fn(&mut Context)> {
    Canvas::default()
        .x_bounds([-200.0, 200.0])
        .y_bounds([-200.0, 200.0])
        .paint(|ctx| {
            //Circle Radius for different circle
            const MAJOR_RADIUS: u16 = 180;
            const MINOR_RADIUS: u16 = 140;
            const ACCIDENTAL_RADIUS: u16 = 160;
            //draw center title
            let x = -25.0;
            let y = 0.0;
            ctx.print(x, y, "Circle of fifth".to_string());
            //draw keys
            let draw_key =
                |ctx: &mut Context, radius: u16, degree: f64, key: &str, color: Color| {
                    let key_len = key.len() as f64;
                    let degree = degree / 180_f64 * std::f64::consts::PI;
                    let y = degree.sin() * (radius as f64);
                    let x = degree.cos() * (radius as f64) - key_len / 2.0;
                    let style = Style::default().fg(color);
                    let text = Span::styled(key.to_string(), style);
                    ctx.print(x, y, text);
                };
            //draw major keys
            let color = Color::Red;
            let degree = 150_f64;
            draw_key(ctx, MAJOR_RADIUS, degree, "Bb", color);
            let degree = 120_f64;
            draw_key(ctx, MAJOR_RADIUS, degree, "F", color);
            let degree = 90_f64;
            draw_key(ctx, MAJOR_RADIUS, degree, "C", color);
            let degree = 60_f64;
            draw_key(ctx, MAJOR_RADIUS, degree, "G", color);
            let degree = 30_f64;
            draw_key(ctx, MAJOR_RADIUS, degree, "D", color);
            let degree = 0_f64;
            draw_key(ctx, MAJOR_RADIUS, degree, "A", color);
            let degree = -30_f64;
            draw_key(ctx, MAJOR_RADIUS, degree, "E", color);
            let degree = -60_f64;
            draw_key(ctx, MAJOR_RADIUS, degree, "B", color);
            let degree = -90_f64;
            draw_key(ctx, MAJOR_RADIUS, degree, "Gb/F#", color);
            let degree = -120_f64;
            draw_key(ctx, MAJOR_RADIUS, degree, "Db", color);
            let degree = -150_f64;
            draw_key(ctx, MAJOR_RADIUS, degree, "Ab", color);
            let degree = -180_f64;
            draw_key(ctx, MAJOR_RADIUS, degree, "Eb", color);
            //draw minor keys
            let color = Color::Blue;
            let degree = 150_f64;
            draw_key(ctx, MINOR_RADIUS, degree, "g", color);
            let degree = 120_f64;
            draw_key(ctx, MINOR_RADIUS, degree, "d", color);
            let degree = 90_f64;
            draw_key(ctx, MINOR_RADIUS, degree, "a", color);
            let degree = 60_f64;
            draw_key(ctx, MINOR_RADIUS, degree, "e", color);
            let degree = 30_f64;
            draw_key(ctx, MINOR_RADIUS, degree, "b", color);
            let degree = 0_f64;
            draw_key(ctx, MINOR_RADIUS, degree, "f#", color);
            let degree = -30_f64;
            draw_key(ctx, MINOR_RADIUS, degree, "c#", color);
            let degree = -60_f64;
            draw_key(ctx, MINOR_RADIUS, degree, "g#", color);
            let degree = -90_f64;
            draw_key(ctx, MINOR_RADIUS, degree, "eb/d#", color);
            let degree = -120_f64;
            draw_key(ctx, MINOR_RADIUS, degree, "bb", color);
            let degree = -150_f64;
            draw_key(ctx, MINOR_RADIUS, degree, "f", color);
            let degree = -180_f64;
            draw_key(ctx, MINOR_RADIUS, degree, "c", color);
            //draw accidentals
            let color = Color::LightYellow;
            let degree = 150_f64;
            draw_key(ctx, ACCIDENTAL_RADIUS, degree, "2b", color);
            let degree = 120_f64;
            draw_key(ctx, ACCIDENTAL_RADIUS, degree, "1b", color);
            let degree = 90_f64;
            draw_key(ctx, ACCIDENTAL_RADIUS, degree, "0", color);
            let degree = 60_f64;
            draw_key(ctx, ACCIDENTAL_RADIUS, degree, "1#", color);
            let degree = 30_f64;
            draw_key(ctx, ACCIDENTAL_RADIUS, degree, "2#", color);
            let degree = 0_f64;
            draw_key(ctx, ACCIDENTAL_RADIUS, degree, "3#", color);
            let degree = -30_f64;
            draw_key(ctx, ACCIDENTAL_RADIUS, degree, "4#", color);
            let degree = -60_f64;
            draw_key(ctx, ACCIDENTAL_RADIUS, degree, "5#", color);
            let degree = -90_f64;
            draw_key(ctx, ACCIDENTAL_RADIUS, degree, "6b/6#", color);
            let degree = -120_f64;
            draw_key(ctx, ACCIDENTAL_RADIUS, degree, "5b", color);
            let degree = -150_f64;
            draw_key(ctx, ACCIDENTAL_RADIUS, degree, "4b", color);
            let degree = -180_f64;
            draw_key(ctx, ACCIDENTAL_RADIUS, degree, "3b", color);
        })
}

//todo: show corresponding main item according to selected item
//todo: draw rthtmic elements, like: note, half note and more
fn ui<B: Backend>(f: &mut Frame<B>, menu_state: &mut MenuState, board_state: &mut BoardState) {
    let boards = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(f.size());

    let menu_boards = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(boards[1]);

    let main_board_block = Block::default().title("Main Board").borders(Borders::ALL);
    let main_board_block = match board_state.current_board {
        Board::Main => main_board_block.border_style(Style::default().fg(Color::Yellow)),
        _ => main_board_block,
    };
    let main_board_content = draw_circle_of_fifths().block(main_board_block);
    f.render_widget(main_board_content, boards[0]);

    let menu_info_board = Block::default().title("Menu Info").borders(Borders::ALL);
    let menu_info_board = match board_state.current_board {
        Board::MenuInfo => menu_info_board.border_style(Style::default().fg(Color::Yellow)),
        _ => menu_info_board,
    };
    let selected_idx = menu_state.state.selected().unwrap();
    let menu_info_text = Text::raw(menu_state.menu_infos.get(selected_idx).unwrap());
    let menu_info_text = Paragraph::new(menu_info_text)
        .block(menu_info_board)
        .scroll((menu_state.info_scrolls[selected_idx], 0))
        .wrap(Wrap { trim: true });
    f.render_widget(menu_info_text, menu_boards[0]);

    let menu_select_block = Block::default().title("Menu Select").borders(Borders::ALL);
    let menu_select_block = match board_state.current_board {
        Board::MenuSelect => menu_select_block.border_style(Style::default().fg(Color::Yellow)),
        _ => menu_select_block,
    };
    let items: Vec<ListItem> = menu_state
        .menu_items
        .iter()
        .map(|item| ListItem::new(item.to_string()))
        .collect();
    let list = List::new(items)
        .block(menu_select_block)
        .highlight_style(Style::default().fg(Color::Yellow));
    f.render_stateful_widget(list, menu_boards[1], &mut menu_state.state);
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
    menu_items: Vec<String>,
    menu_infos: Vec<String>,
    info_scrolls: Vec<u16>,
    state: ListState,
}
impl MenuState {
    fn new() -> Self {
        Self {
            menu_items: vec![],
            menu_infos: vec![],
            state: ListState::default(),
            info_scrolls: vec![],
        }
    }
    fn set_items(&mut self, items: Vec<String>, infos: Vec<String>) {
        assert!(items.len() == infos.len(), "shold be same length");
        self.menu_items = items;
        self.menu_infos = infos;
        self.info_scrolls = vec![0; self.menu_infos.len()];
        self.state.select(Some(0));
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.menu_items.len() - 1 {
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
                    self.menu_items.len() - 1
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
