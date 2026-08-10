use crossterm::event::{self, Event, KeyCode, KeyEventKind, poll};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use ratatui::{Frame, Terminal};
use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;

const LOGO: &str = include_str!("../assets/logo.txt");

const MENU: &[&str] = &[
    "New module",
    "Build",
    "Rebuild automatically",
    "Pack",
    "Search modules on GitHub",
    "Exit",
];

const GRAY: Color = Color::Gray;
const WHITE: Color = Color::White;

struct App {
    screen: usize,
    selected: usize,
    results: Vec<crate::search::Hit>,
    output: Option<(bool, String)>,
    input: Option<(String, String)>,
    status: Option<String>,
}

impl App {
    fn new() -> Self {
        App {
            screen: 0,
            selected: 0,
            results: Vec::new(),
            output: None,
            input: None,
            status: None,
        }
    }

    fn items(&self) -> Vec<String> {
        if self.screen == 1 {
            self.results
                .iter()
                .map(|h| match &h.version {
                    Some(v) => format!("{}  v{v}  \u{2605} {}", h.full, h.stars),
                    None => format!("{}  \u{2605} {}  (no module.json)", h.full, h.stars),
                })
                .collect()
        } else {
            MENU.iter().map(|s| s.to_string()).collect()
        }
    }
}

fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    if area.width < 24 || area.height < 10 {
        f.render_widget(
            Paragraph::new("Terminal too small — enlarge the window"),
            area,
        );
        return;
    }
    let [main, footer] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

    if app.screen == 2 {
        if let Some((ok, text)) = &app.output {
            let w = 72u16.min(main.width.saturating_sub(4));
            let h = (main.height.saturating_sub(2)).min(20);
            let output_area = Rect::new(
                main.x + main.width.saturating_sub(w) / 2,
                main.y + (main.height.saturating_sub(h)) / 2,
                w,
                h,
            );
            let (title, color) = if *ok {
                (" OK ", Color::Green)
            } else {
                (" FAILED ", Color::Red)
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(color))
                .title(Line::from(Span::styled(
                    format!(" OUTPUT {title}"),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )))
                .title_alignment(Alignment::Center);
            let limit = (h as usize).saturating_sub(2);
            let lines: Vec<String> = text
                .trim_end_matches('\n')
                .lines()
                .rev()
                .take(limit)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|s| s.to_string())
                .collect();
            f.render_widget(
                Paragraph::new(lines.join("\n"))
                    .block(block)
                    .style(Style::default().fg(WHITE)),
                output_area,
            );
        }
        let left = "Esc — back to menu    Q — quit".to_string();
        let right = format!("crussty {}", env!("CARGO_PKG_VERSION"));
        let right_w = right.chars().count() as u16;
        let pad = main
            .width
            .saturating_sub(left.chars().count() as u16 + right_w);
        f.render_widget(
            Paragraph::new(
                Line::from(Span::styled(
                    format!("{left}{}{right}", " ".repeat(pad as usize)),
                    Style::default().fg(GRAY),
                )),
            ),
            footer,
        );
        return;
    }

    let logo_w = LOGO.lines().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
    let logo_h = LOGO.lines().count() as u16;
    let items = app.items();

    let menu_w = if app.screen == 1 {
        58u16.min(main.width.saturating_sub(4))
    } else {
        38u16.min(main.width.saturating_sub(4))
    };
    let menu_h = (items.len() as u16 + 2).min(main.height.saturating_sub(2));
    let block_top = if app.screen == 1 {
        main.y + (main.height.saturating_sub(menu_h)) / 2
    } else {
        let block_h = logo_h + 1 + 1 + 2 + menu_h;
        main.y + (main.height.saturating_sub(block_h)) / 2
    };

    if app.screen == 0 {
        for (i, l) in LOGO.lines().enumerate() {
            let y = block_top + i as u16;
            let text: String = l.chars().take(main.width as usize).collect();
            let pad = main.width.saturating_sub(logo_w) / 2;
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("{}{}", " ".repeat(pad as usize), text),
                    Style::default().fg(WHITE),
                ))),
                Rect::new(main.x, y, main.width, 1),
            );
        }
        let title_text = format!(" CRUSSTY ");
        let pad_t = main.width.saturating_sub(title_text.chars().count() as u16) / 2;
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("{}{}", " ".repeat(pad_t as usize), title_text),
                Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
            ))),
            Rect::new(main.x, block_top + logo_h + 1, main.width, 1),
        );
    }

    let menu_area = Rect::new(
        main.x + main.width.saturating_sub(menu_w) / 2,
        block_top + if app.screen == 0 { logo_h + 3 } else { 0 },
        menu_w,
        menu_h,
    );

    let list: Vec<ListItem> = items
        .iter()
        .map(|s| ListItem::new(Line::from(Span::styled(s, Style::default().fg(WHITE)))))
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(GRAY))
        .title(Line::from(Span::styled(
            if app.screen == 1 { " RESULTS " } else { " MENU " },
            Style::default().fg(GRAY),
        )))
        .title_alignment(Alignment::Center);
    let list = List::new(list)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(app.selected));
    f.render_stateful_widget(list, menu_area, &mut state);

    if let Some((prompt, buf)) = &app.input {
        let h = 3u16;
        let w = 44u16.min(main.width.saturating_sub(4));
        let input_area = Rect::new(
            main.x + main.width.saturating_sub(w) / 2,
            main.y + main.height.saturating_sub(h + 2),
            w,
            h,
        );
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Line::from(Span::styled(
                format!(" {prompt} "),
                Style::default().fg(Color::Cyan),
            )));
        let text = format!("{buf}▍");
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                text,
                Style::default().fg(WHITE),
            ))),
            block.inner(input_area),
        );
        f.render_widget(block, input_area);
    }

    let footer_left = if let Some(s) = &app.status {
        s.clone()
    } else if app.screen == 1 {
        "↑/↓ — select    Enter — install    Esc — back    Q — quit".to_string()
    } else {
        "↑/↓ — select    Enter — run    Esc — quit    Q — quit".to_string()
    };
    let footer_right = format!("crussty {}", env!("CARGO_PKG_VERSION"));
    let right_w = footer_right.chars().count() as u16;
    let pad = main
        .width
        .saturating_sub(right_w + footer_left.chars().count() as u16) as usize;
    let left_span = Line::from(vec![
        Span::styled(footer_left, Style::default().fg(GRAY)),
        Span::raw(" ".repeat(pad)),
        Span::styled(footer_right, Style::default().fg(GRAY)),
    ]);
    f.render_widget(Paragraph::new(left_span), footer);
}

fn run_capture(argv: &[String]) -> (bool, String) {
    let exe = std::env::current_exe().unwrap_or_else(|_| "crussty".into());
    match Command::new(&exe).args(argv).output() {
        Ok(out) => {
            let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&out.stderr));
            (out.status.success(), text)
        }
        Err(e) => (false, format!("crussty: {e}")),
    }
}

pub fn run() {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let _ = crossterm::terminal::enable_raw_mode();
    let _ = execute!(stdout, crossterm::cursor::Hide);

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(_) => {
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            let _ = crossterm::terminal::disable_raw_mode();
            return;
        }
    };

    let mut app = App::new();
    loop {
        let _ = terminal.draw(|f| draw(f, &mut app));

        if let Some((_prompt, _buf)) = &app.input {
            match event::read() {
                Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => match k.code {
                    KeyCode::Enter => {
                        let (_p, buf) = app.input.take().unwrap();
                        if app.selected == 4 {
                            let hits = crate::search::fetch_hits(&buf);
                            if hits.is_empty() {
                                app.status = Some(format!("no modules found for '{buf}'"));
                            } else {
                                app.results = hits;
                                app.screen = 1;
                                app.selected = 0;
                            }
                        } else {
                            let argv = match app.selected {
                                0 => vec!["module".into(), "new".into(), buf],
                                _ => vec!["install".into(), buf],
                            };
                            let (ok, text) = run_capture(&argv);
                            app.output = Some((ok, text));
                            app.screen = 2;
                        }
                    }
                    KeyCode::Esc => {
                        app.input = None;
                    }
                    KeyCode::Backspace => {
                        if let Some((_, buf)) = &mut app.input {
                            buf.pop();
                        }
                    }
                    KeyCode::Char(c) => {
                        if let Some((_, buf)) = &mut app.input {
                            if buf.chars().count() < 40 {
                                buf.push(c);
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
            continue;
        }

        let mut advanced = false;
        while let Ok(true) = poll(Duration::from_millis(16)) {
            match event::read() {
                Ok(Event::Key(k))
                    if k.kind == KeyEventKind::Press || k.kind == KeyEventKind::Repeat =>
                {
                    let items_len = app.items().len();
                    match k.code {
                        KeyCode::Up => {
                            app.selected = app.selected.saturating_sub(1);
                        }
                        KeyCode::Down => {
                            app.selected = (app.selected + 1).min(items_len - 1);
                        }
                        KeyCode::Esc => {
                            if app.screen == 1 || app.screen == 2 {
                                app.screen = 0;
                                app.output = None;
                                app.selected = 0;
                            } else {
                                return;
                            }
                        }
                        KeyCode::Enter => {
                            let mut argv: Option<Vec<String>> = None;
                            if app.screen == 1 {
                                if let Some(h) = app.results.get(app.selected) {
                                    argv = Some(vec!["install".into(), h.full.clone()]);
                                } else {
                                    app.screen = 0;
                                    app.selected = 0;
                                }
                            } else {
                                match app.selected {
                                    0 => {
                                        app.input =
                                            Some(("Module name".into(), String::new()))
                                    }
                                    1 => argv = Some(vec!["module".into(), "build".into()]),
                                    2 => argv = Some(vec!["module".into(), "watch".into()]),
                                    3 => argv = Some(vec!["module".into(), "pack".into()]),
                                    4 => app.input = Some(("Search".into(), String::new())),
                                    _ => return,
                                }
                            }
                            if let Some(argv) = argv {
                                if argv.get(1).map(|s| s.as_str()) == Some("watch") {
                                    advanced = true;
                                    let _ = execute!(
                                        io::stdout(),
                                        LeaveAlternateScreen,
                                        crossterm::cursor::Show
                                    );
                                    let _ = crossterm::terminal::disable_raw_mode();
                                    let _ = io::stdout().flush();
                                    let exe = std::env::current_exe()
                                        .unwrap_or_else(|_| "crussty".into());
                                    let code = Command::new(&exe)
                                        .args(&argv)
                                        .status()
                                        .map(|s| s.code().unwrap_or(1))
                                        .unwrap_or(1);
                                    let _ = execute!(
                                        io::stdout(),
                                        EnterAlternateScreen,
                                        crossterm::cursor::Hide
                                    );
                                    let _ = crossterm::terminal::enable_raw_mode();
                                    let _ = terminal.clear();
                                    app.status = Some(if code == 0 {
                                        "✓ done".to_string()
                                    } else {
                                        format!("✗ exited with code {code}")
                                    });
                                } else {
                                    let (ok, text) = run_capture(&argv);
                                    app.output = Some((ok, text));
                                    app.screen = 2;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            if advanced {
                break;
            }
        }
    }
}

