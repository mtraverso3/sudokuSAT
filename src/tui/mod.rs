use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs};
use std::io::{self, stdout};
use std::time::{Duration, Instant};

use crate::solver::{SolverKind, SudokuSolver, make_solver};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Focus {
    Solver,
    Grid,
}

struct App {
    grid: [[usize; 9]; 9],
    cursor: (usize, usize),
    solver_idx: usize, // 0 = SAT, 1 = Backtracking, 2 = ExactCover (not yet implemented)
    focus: Focus,
    message: Option<String>,
    show_help: bool,
    last_solve_time: Option<Duration>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            grid: [[0; 9]; 9],
            cursor: (0, 0),
            solver_idx: 0,
            focus: Focus::Grid,
            message: None,
            show_help: true,
            last_solve_time: None,
        }
    }
}

fn default_puzzle() -> [[usize; 9]; 9] {
    [
        [0, 3, 6, 0, 0, 0, 9, 0, 0],
        [1, 0, 0, 5, 3, 0, 2, 0, 0],
        [0, 0, 4, 0, 0, 0, 0, 0, 6],
        [0, 4, 7, 0, 0, 0, 0, 5, 3],
        [0, 0, 0, 0, 0, 8, 0, 6, 9],
        [6, 9, 0, 0, 4, 0, 0, 0, 0],
        [0, 0, 0, 8, 0, 7, 0, 0, 1],
        [0, 0, 2, 0, 0, 0, 0, 0, 4],
        [0, 8, 5, 0, 0, 0, 0, 2, 0],
    ]
}

fn solver_titles() -> Vec<Line<'static>> {
    vec!["SAT", "Backtracking", "ExactCover"]
        .into_iter()
        .map(|t| Line::from(t.to_string()))
        .collect()
}

fn current_solver_kind(idx: usize) -> SolverKind {
    match idx {
        0 => SolverKind::Sat,
        1 => SolverKind::Backtracking,
        // 2 => SolverKind::ExactCover,
        _ => SolverKind::Sat,
    }
}

pub fn run() -> io::Result<()> {
    // Setup terminal in raw mode and alternate screen
    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();

    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    // Propagate error if any
    if let Err(e) = res {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if handle_key(app, key)? {
                    break; // exit
                }
            }
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    // When help is visible, only toggle/close help or quit
    if app.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('h') => {
                app.show_help = false;
            }
            KeyCode::Char('q') => return Ok(true),
            _ => {}
        }
        return Ok(false);
    }

    match key.code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Char('?') | KeyCode::Char('h') => {
            app.show_help = true;
        }
        KeyCode::Char('d') => {
            app.grid = default_puzzle();
            app.message = Some("Loaded default puzzle".into());
            app.cursor = (0, 0);
        }
        KeyCode::Tab => {
            app.focus = match app.focus {
                Focus::Grid => Focus::Solver,
                Focus::Solver => Focus::Grid,
            };
        }
        KeyCode::Char('s') => {
            app.message = Some("Solving...".into());
            let kind = current_solver_kind(app.solver_idx);
            let mut solver = make_solver(kind);
            let start = Instant::now();
            match solver.solve(&app.grid) {
                Some(sol) => {
                    app.grid = sol;
                    let elapsed = start.elapsed();
                    app.last_solve_time = Some(elapsed);
                    app.message = Some(format!("Solved in {} ms", elapsed.as_millis()));
                }
                None => {
                    let elapsed = start.elapsed();
                    app.last_solve_time = Some(elapsed);
                    app.message = Some(format!("No solution ({} ms)", elapsed.as_millis()));
                }
            }
        }
        KeyCode::Char('c') => {
            app.grid = [[0; 9]; 9];
            app.message = Some("Cleared grid".into());
            app.last_solve_time = None;
        }
        _ => match app.focus {
            Focus::Grid => handle_grid_keys(app, key),
            Focus::Solver => handle_solver_keys(app, key),
        },
    }
    Ok(false)
}

fn handle_solver_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Left => {
            if app.solver_idx > 0 {
                app.solver_idx -= 1;
            }
        }
        KeyCode::Right => {
            if app.solver_idx + 1 < solver_titles().len() {
                app.solver_idx += 1;
            }
        }
        KeyCode::Char('0') => app.solver_idx = 0,
        KeyCode::Char('1') => app.solver_idx = 1.min(solver_titles().len() - 1),
        KeyCode::Char('2') => app.solver_idx = 2.min(solver_titles().len() - 1),
        KeyCode::Enter => app.focus = Focus::Grid,
        _ => {}
    }
}

fn handle_grid_keys(app: &mut App, key: KeyEvent) {
    let (mut r, mut c) = app.cursor;
    match key.code {
        KeyCode::Up => {
            if r > 0 {
                r -= 1;
            }
        }
        KeyCode::Down => {
            if r < 8 {
                r += 1;
            }
        }
        KeyCode::Left => {
            if c > 0 {
                c -= 1;
            }
        }
        KeyCode::Right => {
            if c < 8 {
                c += 1;
            }
        }
        KeyCode::Char(ch) if ch.is_ascii_digit() => {
            let d = (ch as u8 - b'0') as usize;
            app.grid[r][c] = d;
        }
        KeyCode::Backspace | KeyCode::Delete => {
            app.grid[r][c] = 0;
        }
        _ => {}
    }
    app.cursor = (r, c);
}

fn ui(f: &mut ratatui::Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Min(10),   // grid
            Constraint::Length(2), // status
        ])
        .split(f.size());

    // Tabs for solver selection
    let titles = solver_titles();
    let tabs = Tabs::new(titles)
        .select(app.solver_idx)
        .block(Block::default().title("Solver").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    // Grid drawing; include last solve time in the title if available
    let grid_title = if let Some(t) = app.last_solve_time {
        format!("Sudoku  —  Last: {} ms", t.as_millis())
    } else {
        "Sudoku".to_string()
    };
    let grid_block = Block::default().title(grid_title).borders(Borders::ALL);
    let lines = render_grid_lines(&app.grid, app.cursor);
    let para = Paragraph::new(lines).block(grid_block);
    f.render_widget(para, chunks[1]);

    // Status/help section with right-aligned time indicator
    let status_outer = Block::default()
        .borders(Borders::ALL)
        .title(match app.focus {
            Focus::Grid => "Focus: Grid",
            Focus::Solver => "Focus: Solver",
        });
    f.render_widget(status_outer.clone(), chunks[2]);
    let inner = status_outer.inner(chunks[2]);
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(inner);

    let left_status = app.message.clone().unwrap_or_else(|| {
        "Tab: focus • Arrows/0-9: edit • s: solve • d: default • c: clear • q: quit • ?: help"
            .to_string()
    });
    let left_para = Paragraph::new(Line::from(left_status));
    f.render_widget(left_para, status_chunks[0]);

    if let Some(t) = app.last_solve_time {
        let right_para = Paragraph::new(Line::from(format!("Last solve: {} ms", t.as_millis())))
            .alignment(Alignment::Right);
        f.render_widget(right_para, status_chunks[1]);
    }

    // Draw help overlay last so it sits on top
    if app.show_help {
        let area = centered_rect(80, 80, f.size());
        let help_lines = vec![
            Line::from(Span::styled("Controls", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Global", Style::default().fg(Color::Yellow))),
            Line::from("  q: quit    ?,h: toggle help"),
            Line::from(""),
            Line::from(Span::styled("Focus", Style::default().fg(Color::Yellow))),
            Line::from("  Tab: switch focus between Grid and Solver tabs"),
            Line::from(""),
            Line::from(Span::styled("Grid editing", Style::default().fg(Color::Yellow))),
            Line::from("  Arrows: move cursor    0-9: set cell (0 clears)"),
            Line::from("  Backspace/Delete: clear current cell"),
            Line::from("  c: clear entire grid    s: solve with selected solver"),
            Line::from("  d: load sample default puzzle"),
            Line::from(""),
            Line::from(Span::styled("Solver selection", Style::default().fg(Color::Yellow))),
            Line::from("  Left/Right: change solver tab"),
            Line::from("  0/1/2: jump to specific solver    Enter: back to Grid"),
            Line::from(""),
            Line::from("SAT and Backtracking are implemented; ExactCover coming soon."),
            Line::from("The last solve time is shown in the Sudoku title and the status bar."),
            Line::from("Press Esc, ? or h to close this help."),
        ];
        let help =
            Paragraph::new(help_lines).block(Block::default().title("Help").borders(Borders::ALL));
        f.render_widget(Clear, area); // clear area beneath overlay
        f.render_widget(help, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1]);
    horiz[1]
}

fn render_grid_lines(grid: &[[usize; 9]; 9], cursor: (usize, usize)) -> Vec<Line<'static>> {
    let mut lines = Vec::with_capacity(13);
    for r in 0..9 {
        if r > 0 && r % 3 == 0 {
            lines.push(Line::from("------+-------+------"));
        }
        let mut spans: Vec<Span> = Vec::with_capacity(20);
        for c in 0..9 {
            if c > 0 {
                if c % 3 == 0 {
                    spans.push(Span::raw("| "));
                } else {
                    spans.push(Span::raw(""));
                }
            }
            let val = grid[r][c];
            let ch = if val == 0 {
                '.'
            } else {
                char::from(b'0' + val as u8)
            };
            let mut span = Span::raw(format!("{} ", ch));
            if (r, c) == cursor {
                span.style = Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD);
            }
            spans.push(span);
        }
        lines.push(Line::from(spans));
    }
    lines
}
