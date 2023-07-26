/**
 * Central Limit
 *
 * A simulation of the Central Limit Theorem.
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
    symbols,
    text::Span,
    widgets::{Axis, BarChart, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame, Terminal,
};

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
        let (b_min, b_max) = (-(self.r_max + 2), self.r_max + 2);

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
            let sum = sums.iter().filter(|s| *s == &b).count() as u64;
            self.data.push((format!("{}", b), sum));
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(500);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

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
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(40),
                Constraint::Percentage(40),
            ]
            .as_ref(),
        )
        .split(f.size());

    f.render_widget(
        Paragraph::new(format!(
            "A simulation of the Central Limit Theorem\n\nAfaan Bilal | https://afaan.dev\n\nIterations per render: {} | Tick rate: {}ms | Buckets: {}\nInspired by this excellent 3B1B video: https://youtu.be/zeJD6dqJ5lo\nPress q to quit",
            &app.b_count, 500, &app.r_max
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

    let bar_data = app
        .data
        .iter()
        .map(|x| (x.0.as_str(), x.1))
        .collect::<Vec<_>>();

    let bar_chart = BarChart::default()
        .block(Block::default().borders(Borders::ALL))
        .data(&bar_data)
        .bar_width(7)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::Green))
        .label_style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        )
        .value_style(Style::default().fg(Color::Black).bg(Color::Green));
    f.render_widget(bar_chart, chunks[1]);

    let app_line_data = app
        .data
        .iter()
        .map(|x| (x.0.parse::<f64>().unwrap(), x.1 as f64))
        .collect::<Vec<_>>();

    let line_data = vec![Dataset::default()
        .marker(symbols::Marker::Dot)
        .style(Style::default().fg(Color::Yellow))
        .graph_type(GraphType::Line)
        .data(&app_line_data)];

    let y_max = (app.b_count as f64) / 4.5;

    let chart = Chart::new(line_data)
        .block(Block::default().borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::styled(
                        format!("-{}", app.r_max),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("0"),
                    Span::styled(
                        format!("{}", app.r_max),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ])
                .bounds([-app.r_max as f64, app.r_max as f64]),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!("{:.0}", y_max),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ])
                .bounds([0.0, y_max]),
        );
    f.render_widget(chart, chunks[2]);
}
