use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::terminal;
use crossterm::tty::IsTty;
use std::io::{self, Write};
use std::time::Duration;

const LOGO: &str = include_str!("../assets/logo.txt");
const LOGO_LINES: usize = 6;

const MENU: &[&str] = &[
    "Запустить сервер",
    "Создать сервер (init)",
    "Журнал сервера (log)",
    "Остановить сервер",
    "Список модулей (ls)",
    "Работа с модулями",
    "Поиск модулей на GitHub",
    "Установить модуль",
    "Выход",
];

const SUBMENU: &[&str] = &[
    "Новый модуль (new)",
    "Собрать (build)",
    "Пересборка при изменениях (watch)",
    "Упаковать (pack)",
    "Назад",
];

pub enum Action {
    Run,
    Stop,
    Log,
    Ls,
    Init { dir: String },
    ModuleNew { name: String },
    ModuleBuild,
    ModuleWatch,
    ModulePack,
    Search { query: String },
    Install { module: String },
    Quit,
}

fn logo_width() -> usize {
    LOGO.lines().map(|l| l.chars().count()).max().unwrap_or(0)
}

fn layout(items: usize) -> (u16, u16, u16) {
    let (cols, rows) = size().unwrap_or((80, 24));
    let logo_w = logo_width() as u16;
    let menu_h = items as u16 + 4;
    let y_logo = rows.saturating_sub(menu_h + LOGO_LINES as u16 + 8) / 2;
    let x_center = cols.saturating_sub(logo_w) / 2;
    let x_menu = cols.saturating_sub(30) / 2;
    (x_center, y_logo, x_menu)
}

fn render_menu(out: &mut impl Write, items: &[&str], selected: usize) {
    let (cols, rows) = size().unwrap_or((80, 24));
    let (x_center, y_logo, x_menu) = layout(items.len());

    let _ = execute!(out, SetBackgroundColor(Color::Black), SetForegroundColor(Color::White), Clear(ClearType::All), MoveTo(0, 0));

    for (i, line) in LOGO.lines().enumerate() {
        let _ = execute!(out, MoveTo(x_center, y_logo + i as u16), Print(line));
    }

    for (i, item) in items.iter().enumerate() {
        let y = y_logo + LOGO_LINES as u16 + 4 + i as u16;
        if i == selected {
            let _ = execute!(
                out,
                MoveTo(x_menu, y),
                SetBackgroundColor(Color::White),
                SetForegroundColor(Color::Black),
                Print(format!("  {}  ", item)),
                ResetColor
            );
        } else {
            let _ = execute!(
                out,
                MoveTo(x_menu, y),
                SetBackgroundColor(Color::Black),
                SetForegroundColor(Color::White),
                Print(format!("  {}  ", item))
            );
        }
    }

    let hint = "↑/↓ — выбор   Enter — ок   Q — выход";
    let _ = execute!(
        out,
        MoveTo(cols.saturating_sub(hint.chars().count() as u16) / 2, rows.checked_sub(2).unwrap_or(0)),
        Print(hint),
        MoveTo(0, 0)
    );
}

fn wait_key() -> KeyCode {
    loop {
        match event::read() {
            Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => return k.code,
            Ok(Event::Resize(_, _)) => return KeyCode::Null,
            _ => {}
        }
    }
}

fn ask_input(out: &mut impl Write, prompt: &str) -> Option<String> {
    let (cols, rows) = size().unwrap_or((80, 24));
    let mut buf = String::new();
    loop {
        let label = format!("{}: {}", prompt, buf);
        let _ = execute!(
            out,
            MoveTo(0, rows / 2),
            SetForegroundColor(Color::White),
            SetBackgroundColor(Color::Black),
            Clear(ClearType::CurrentLine),
            Print(&label),
            MoveTo(0, 0)
        );
        let _ = out.flush();
        match wait_key() {
            KeyCode::Enter => return Some(buf),
            KeyCode::Esc => return None,
            KeyCode::Backspace => {
                buf.pop();
            }
            KeyCode::Char(c) => {
                if label.chars().count() < cols as usize {
                    buf.push(c);
                }
            }
            _ => {}
        }
    }
}

fn exit_screen(out: &mut impl Write) {
    let _ = execute!(out, LeaveAlternateScreen, Show, ResetColor);
    let _ = terminal::disable_raw_mode();
}

pub fn run() -> Option<Action> {
    let mut out = io::stdout();
    let _ = execute!(out, EnterAlternateScreen);
    let _ = terminal::enable_raw_mode();
    let _ = execute!(
        out,
        SetBackgroundColor(Color::Black),
        SetForegroundColor(Color::White),
        Hide
    );

    let (x_center, y_logo, _) = layout(MENU.len());
    let _ = execute!(out, Clear(ClearType::All), MoveTo(0, 0));
    for (i, line) in LOGO.lines().enumerate() {
        let _ = execute!(out, MoveTo(x_center, y_logo + i as u16), Print(line));
        let _ = out.flush();
        std::thread::sleep(Duration::from_millis(110));
    }
    std::thread::sleep(Duration::from_millis(450));

    let mut screen = 0usize;
    let mut selected = 0usize;
    loop {
        let items = if screen == 0 { MENU } else { SUBMENU };
        render_menu(&mut out, items, selected);
        let _ = out.flush();
        let key = wait_key();
        match key {
            KeyCode::Up => {
                selected = selected.saturating_sub(1);
            }
            KeyCode::Down => {
                selected = (selected + 1).min(items.len() - 1);
            }
            KeyCode::Char('q') | KeyCode::Esc => break,
            KeyCode::Enter => {
                let action = match screen {
                    0 => match selected {
                        0 => Some(Action::Run),
                        1 => {
                            let dir = ask_input(&mut out, "Каталог сервера (Enter — текущий)");
                            match dir {
                                Some(dir) => {
                                    let dir: String = if dir.is_empty() { ".".into() } else { dir };
                                    Some(Action::Init { dir })
                                }
                                None => None,
                            }
                        }
                        2 => Some(Action::Log),
                        3 => Some(Action::Stop),
                        4 => Some(Action::Ls),
                        5 => {
                            screen = 1;
                            selected = 0;
                            None
                        }
                        6 => ask_input(&mut out, "Поиск").map(|query| Action::Search { query }),
                        7 => ask_input(&mut out, "Модуль (id или owner/repo)").map(|module| Action::Install { module }),
                        _ => Some(Action::Quit),
                    },
                    _ => match selected {
                        0 => ask_input(&mut out, "Имя модуля").map(|name| Action::ModuleNew { name }),
                        1 => Some(Action::ModuleBuild),
                        2 => Some(Action::ModuleWatch),
                        3 => Some(Action::ModulePack),
                        _ => {
                            screen = 0;
                            None
                        }
                    },
                };
                if let Some(action) = action {
                    exit_screen(&mut out);
                    return Some(action);
                }
            }
            _ => {}
        }
    }

    exit_screen(&mut out);
    Some(Action::Quit)
}

pub fn needs_tty() -> bool {
    io::stdin().is_tty()
}