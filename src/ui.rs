use crate::app::{App, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

fn state_color(active: &str, sub: &str) -> Color {
    match (active, sub) {
        ("active", "running") => Color::Green,
        ("active", _) => Color::LightGreen,
        ("failed", _) => Color::Red,
        ("inactive", "dead") => Color::DarkGray,
        ("activating", _) => Color::Yellow,
        _ => Color::White,
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Tabs = ecosistema de tu diagrama
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| Line::from(Span::styled(t.title(), Style::default().fg(Color::White))))
        .collect();
    let idx = Tab::all()
        .iter()
        .position(|t| *t == app.active_tab)
        .unwrap_or(0);
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("ksys · systemd como ecosistema · BigLinux/Arch/Deb/Fedora"),
        )
        .select(idx)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    // Izquierda: lista units
    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .enumerate()
        .map(|(list_idx, unit_idx)| {
            let u = &app.units[*unit_idx];
            let dot = match u.active_state.as_str() {
                "active" => "●",
                "failed" => "✗",
                _ => "○",
            };
            let c = state_color(&u.active_state, &u.sub_state);
            let sel = list_idx == app.selected;
            let line = Line::from(vec![
                Span::styled(
                    format!("{dot} "),
                    Style::default().fg(c).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<38} {:<10}", u.name, u.sub_state),
                    if sel {
                        Style::default().fg(Color::Black).bg(Color::Green)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]);
            ListItem::new(line)
        })
        .collect();
    let title_left = format!(
        "{} · filtro: {} (/)",
        app.active_tab.title(),
        if app.filter.is_empty() {
            "—"
        } else {
            &app.filter
        }
    );
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title_left));
    f.render_widget(list, mid[0]);

    // Derecha: detail + journal
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(5)])
        .split(mid[1]);
    let detail = Paragraph::new(app.detail.clone()).block(
        Block::default()
            .borders(Borders::ALL)
            .title("detail · systemctl show"),
    );
    f.render_widget(detail, right[0]);

    let log_title = if app.follow {
        "journal · FOLLOW (f)"
    } else {
        "journal · journalctl (f=follow)"
    };
    let log = Paragraph::new(app.journal_lines.join("\n"))
        .block(Block::default().borders(Borders::ALL).title(log_title));
    f.render_widget(log, right[1]);

    // Footer: status + confirm
    let footer_text = if let Some(p) = &app.confirm {
        format!(
            "⚠ {} {} ?  [y] si  [n] no   (polkit pedira auth solo por esta accion)",
            p.action, p.unit
        )
    } else {
        app.status.clone()
    };
    let style = if app.confirm.is_some() {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .style(style);
    f.render_widget(footer, chunks[2]);
}
