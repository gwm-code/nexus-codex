use std::io::{self, Stdout};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use crate::{
    cache::CacheState,
    storage::{cache_path, load_cache},
    Config,
};

pub fn run(config: &Config) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = ui_loop(&mut terminal, config);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

pub fn run_diff(root: &str) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = diff_loop(&mut terminal, root);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn ui_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, config: &Config) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let size = frame.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(size);

            let header = Paragraph::new(Line::from("Nexus CLI - Phase 1"))
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL).title("Status"));
            frame.render_widget(header, chunks[0]);

            let body = Paragraph::new(vec![
                Line::from(format!("Provider: {:?}", config.provider)),
                Line::from(format!("Dry run: {}", config.dry_run)),
                Line::from(format!(
                    "Config path: {}",
                    Config::path()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "n/a".to_string())
                )),
                Line::from("Press q to exit."),
            ])
            .block(Block::default().borders(Borders::ALL).title("Config"));
            frame.render_widget(body, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn diff_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, root: &str) -> io::Result<()> {
    let cached = load_cache(
        cache_path()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?
            .as_path(),
    )
    .unwrap_or_default();
    let mut current = CacheState::new(root.into());
    let _ = current.warm();
    let diff = cached.diff(&current);

    let changed_items: Vec<ListItem> = diff
        .changed
        .iter()
        .map(|item| ListItem::new(item.clone()))
        .collect();
    let removed_items: Vec<ListItem> = diff
        .removed
        .iter()
        .map(|item| ListItem::new(item.clone()))
        .collect();

    loop {
        terminal.draw(|frame| {
            let size = frame.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(size);

            let header = Paragraph::new(Line::from(format!(
                "Nexus Diff Viewer • root={}",
                root
            )))
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title("Diff"));
            frame.render_widget(header, chunks[0]);

            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            let changed = List::new(changed_items.clone())
                .block(Block::default().borders(Borders::ALL).title("Changed"));
            frame.render_widget(changed, columns[0]);

            let removed = List::new(removed_items.clone())
                .block(Block::default().borders(Borders::ALL).title("Removed"));
            frame.render_widget(removed, columns[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    Ok(())
}
