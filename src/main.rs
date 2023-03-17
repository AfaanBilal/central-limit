/**
 * Central Limit
 *
 * A simple terminal UI demo of the central limit theorem.
 *
 * @author      Afaan Bilal
 * @link        https://afaan.dev
 *
 */
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::prelude::*;
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{BarChart, Block, Borders, Paragraph},
    Frame, Terminal,
};

const BUCKET_PADDING: i32 = 3;

struct App {
    b_count: usize,
    r_max: i32,
    data: Vec<(String, u64)>,
}

impl App {
    fn new() -> App {
        App {
            b_count: 5000,
            r_max: 19, // must be odd
            data: vec![],
        }
    }

    fn on_tick(&mut self) {
        let (b_min, b_max) = (-self.r_max, self.r_max + BUCKET_PADDING);

        let mut buckets = vec![];
        for r in b_min..=b_max {
            if r % 2 != 0 {
                buckets.push(r);
            }
        }

        let mut sums = vec![];
        for b in 0..self.b_count {
            sums.push(0);
            sums[b] = 0;
            for _ in 0..self.r_max {
                if thread_rng().gen_range(0..10) < 5 {
                    sums[b] -= 1;
                } else {
                    sums[b] += 1;
                }
            }
        }

        self.data.clear();

        for b in buckets {
            self.data.push((
                format!("{}", b),
                sums.iter().filter(|s| *s == &b).count() as u64,
            ));
        }
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
    let tick_rate = Duration::from_millis(500);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(15), Constraint::Percentage(85)].as_ref())
        .split(f.size());

    f.render_widget(
        Paragraph::new(format!(
            "\n\nCentral Limit Theorem - A simple TUI demo\n\n\n Afaan Bilal | https://afaan.dev\n\nIterations per render: {} | Buckets: {}",
            &app.b_count, &app.r_max
        ))
        .style(
            Style::default()
                .bg(Color::Black)
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center),
        chunks[0],
    );

    let app_data = app
        .data
        .iter()
        .map(|x| (x.0.as_str(), x.1))
        .collect::<Vec<_>>();

    let bar_chart = BarChart::default()
        .block(Block::default().borders(Borders::ALL))
        .data(&app_data)
        .bar_width(7)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::Green))
        .label_style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        )
        .value_style(Style::default().fg(Color::White).bg(Color::Green));
    f.render_widget(bar_chart, chunks[1]);
}
