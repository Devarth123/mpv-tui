use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    style::style,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{collections::HashMap, thread, time};
use std::{error::Error, io, process};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame, Terminal,
};
enum InputMode {
    Normal,
    Editing,
    PopupM,
}
#[derive(PartialEq)]
enum Popup {
    SizeErr,
    HelpMenu,
    QuickSettings,
    Close,
}
/// A
///
struct Settings {
    hyprland_support: bool,
    no_video: bool,
    render_again: bool,
}

impl Settings {
    fn default() -> Self {
        Settings {
            hyprland_support: true,
            no_video: false,
            render_again: false,
        }
    }
}

struct App {
    buff: String,

    show_popup: Popup,

    input_mode: InputMode,

    history_buff: Vec<String>,

    playlist: HashMap<u32, String>,

    settings: Settings,

    dir: String,

    center_rect: Rect,

    runder: bool,
}

impl App {
    fn new() -> Self {
        App {
            buff: String::new(),
            show_popup: Popup::Close,
            input_mode: InputMode::Normal,
            history_buff: Vec::new(),
            playlist: HashMap::new(),
            settings: Settings::default(),
            dir: String::from("~/Music/"),
            center_rect: Rect::default(),
            runder: false,
        }
    }
    fn close_popups<B: Backend>(&self, f: &mut Frame<B>) {
        f.render_widget(Clear, self.center_rect);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = init(&mut terminal)?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // if let Err(err) = res {
    //     println!("{:?}", err)
    // }

    Ok(())
}

fn init<B: Backend>(term: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new();
    let mut counter = 1;
    loop {
        term.draw(|f| {
            // app.center_rect = centered_rect(60, 20, term.size().unwrap());
            // if let Ok(size) = term.size() {
            //     if size.height < 20 || size.width < 20 {
            //         app.show_popup = Popup::SizeErr;
            //     }
            //     if size.height > 20 && size.width > 20 {
            //         app.close_popups(f)
            //     }
            // }

            ui(f, &app);
        })?;

        let mut f = term.get_frame();
        app.center_rect = centered_rect(60, 20, f.size());
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('i') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('s') => {
                        app.runder = true;
                        app.input_mode = InputMode::PopupM;
                        f.render_widget(settings(&app), app.center_rect);
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Char(c) => {
                        app.buff.push(c);
                    }
                    KeyCode::Up => {
                        if counter != 0 && app.history_buff.len() >= counter {
                            app.buff = app.history_buff[app.history_buff.len() - counter].clone();
                            counter += 1;
                        }
                    }
                    KeyCode::Enter => {
                        app.history_buff.push(app.buff.drain(..).collect());
                    }
                    KeyCode::Backspace => {
                        app.buff.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::PopupM => match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        f.render_widget(Clear, app.center_rect);
                        app.input_mode = InputMode::Normal;
                    }

                    KeyCode::Char('h') => {
                        app.settings.hyprland_support = !app.settings.hyprland_support;
                        f.render_widget(settings(&app), app.center_rect);
                    }

                    KeyCode::Char('n') => {
                        app.settings.no_video = !app.settings.no_video;
                        f.render_widget(settings(&app), app.center_rect);
                    }
                    _ => {}
                },
            }
        }
    }
}

fn settings(app: &App) -> Paragraph {
    Paragraph::new(format!(
        "Hyprland Support -> {}\n\nNo video        -> {}",
        app.settings.hyprland_support,
        app.settings.no_video
    ))
    .block(
        Block::default()
            .style(Style::default().fg(Color::LightGreen))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    )
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());
    if app.show_popup == Popup::SizeErr {
        let block = Paragraph::new("Too Small! (thats what she said)")
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(Clear, app.center_rect); //this clears out the background

        f.render_widget(block, app.center_rect);
    }

    let search_bar = Paragraph::new(app.buff.as_ref())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .style(Style::default().fg(Color::White))
                .border_style(match app.input_mode {
                    InputMode::Normal => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::LightGreen),
                    InputMode::PopupM => Style::default(),
                })
                .border_type(BorderType::Rounded),
        );
    match app.input_mode {
        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[2].x + app.buff.len() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[2].y + 2,
            )
        }
        _ => {}
    }

    f.render_widget(
        search_bar,
        Rect {
            x: chunks[2].x,
            y: chunks[2].y + 1,
            width: chunks[2].width,
            height: (chunks[2].height as f64 / 1.2) as u16,
        },
    );

    let playlist_box = Block::default()
        .border_type(BorderType::Rounded)
        .title("Songs")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .border_style(match app.input_mode {
            InputMode::Normal => Style::default().fg(Color::Green),
            InputMode::Editing => Style::default(),
            InputMode::PopupM => Style::default(),
        });
    f.render_widget(
        playlist_box,
        Rect {
            x: 0,
            y: 0,
            width: chunks[1].width + 0,
            height: chunks[1].height + 4,
        },
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
