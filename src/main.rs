mod app;
mod journal;
mod systemd;
mod ui;

use anyhow::Result;
use app::{App, Tab};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

#[derive(Parser, Debug)]
#[command(
    name = "ksys",
    version,
    about = "lazysystemd: ecosistema systemd en TUI (sin sudo total)"
)]
struct Cli {
    /// Tipo inicial: service, timer, socket...
    #[arg(long, default_value = "service")]
    kind: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut app = App::new();
    if cli.kind == "timer" {
        app.active_tab = Tab::Timers;
    }
    app.refresh_units().await;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("ksys error: {e:#}");
    }
    Ok(())
}

async fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                // Confirm modal takes over
                if app.confirm.is_some() {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            let action = app.confirm.take();
                            if let Some(a) = action {
                                app.execute_pending(a).await;
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.confirm = None;
                            app.status = "cancelado".to_string();
                        }
                        _ => {}
                    }
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(())
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                    KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::BackTab => app.prev_tab(),
                    KeyCode::Char('1') => app.set_tab(Tab::Units),
                    KeyCode::Char('2') => app.set_tab(Tab::Timers),
                    KeyCode::Char('3') => app.set_tab(Tab::Journal),
                    KeyCode::Char('4') => app.set_tab(Tab::Boot),
                    KeyCode::Char('5') => app.set_tab(Tab::Network),
                    KeyCode::Char('6') => app.set_tab(Tab::Sessions),
                    KeyCode::Char('7') => app.set_tab(Tab::Devices),
                    KeyCode::Char('/') => app.toggle_filter_input(),
                    KeyCode::Enter => {
                        if app.filtering {
                            app.apply_filter_and_close().await;
                        }
                    }
                    KeyCode::Char(c) if app.filtering => app.push_filter(c),
                    KeyCode::Backspace if app.filtering => app.pop_filter(),
                    // Acciones estilo lazygit: s/t/r/e/d (start/stop/restart/enable/disable)
                    KeyCode::Char('s') => app.request_action("start").await,
                    KeyCode::Char('t') => app.request_action("stop").await,
                    KeyCode::Char('r') => app.request_action("restart").await,
                    KeyCode::Char('e') => app.request_action("enable").await,
                    KeyCode::Char('d') => app.request_action("disable").await,
                    KeyCode::Char('f') => app.toggle_follow().await,
                    _ => {}
                }
            }
        }
        // Poll liviano del journal si follow esta activo
        app.maybe_poll_follow().await;
    }
}
