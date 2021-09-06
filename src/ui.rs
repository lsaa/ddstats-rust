//
// Funny UI
//

use std::io::Stdout;

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Corner, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};

use crate::mem::StatsBlockWithFrames;

pub fn create_term() -> Terminal<CrosstermBackend<Stdout>> {
    Terminal::new(CrosstermBackend::new(std::io::stdout())).expect("Funny terminal")
}

pub fn draw_logo<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let logo: Vec<&str> = crate::consts::LOGO_NEW.lines().collect();
    let logo: Vec<Spans> = logo
        .into_iter()
        .map(|string| {
            Spans::from(vec![Span::styled(
                string,
                Style::default().add_modifier(Modifier::BOLD),
            )])
        })
        .collect();

    let paragraph = Paragraph::new(logo)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(Color::Red).bg(Color::Black))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

pub fn draw_info_table<B>(f: &mut Frame<B>, area: Rect, last_data: &StatsBlockWithFrames)
where
    B: Backend,
{
    let real_timer = last_data.block.time_max + last_data.block.starting_time;
    let normal_style = Style::default().fg(Color::White);
    let funny_ghost_times = [366., 709., 800., 875., 942., 996., 1047., 1091., 1133.];
    let total_gems_lost = last_data.block.gems_eaten + last_data.block.gems_despawned;
    let collection_acc =
        (last_data.block.gems_total - total_gems_lost) as f32 / last_data.block.gems_total as f32;

    let user_data = format!("{}", last_data.block.replay_player_username());
    let timer = format!("{:.4}s", real_timer);
    let stt = format!("{}", "REPLAY");
    let gems = format!("{}", last_data.block.gems_total);
    let kills = format!("{}", last_data.block.kills);
    let gems_lost = format!(
        "{} [{} DESPAWNED; {} EATEN]",
        total_gems_lost, last_data.block.gems_despawned, last_data.block.gems_eaten
    );
    let acc = format!(
        "{:.1}%",
        (last_data.block.daggers_hit as f32 / last_data.block.daggers_fired as f32) * 100.0
    );
    let homing = format!(
        "{} [MAX {} at {:.4}s]",
        last_data.block.homing, last_data.block.max_homing, last_data.block.time_max_homing
    );
    let colection_accuracy = format!("{:.1}%", collection_acc * 100.0);

    let header = [stt.as_str(), user_data.as_str()];

    let mut splits = Vec::new();
    for time in funny_ghost_times {
        if time < real_timer {
            if let Some(time_frame) = last_data.get_frame_for_time(time) {
                splits.push(Row::new(vec!["SPLIT".to_owned(), format!("{} HOMING AT {:.1}s", time_frame.homing, time)]).style(normal_style));
            }
        }
    }

    let mut rows = vec![
        Row::new(vec!["TIMER", timer.as_str()]).style(normal_style),
        Row::new(vec!["GEMS", gems.as_str()]).style(normal_style),
        Row::new(vec!["HOMING", homing.as_str()]).style(normal_style),
        Row::new(vec!["KILLS", kills.as_str()]).style(normal_style),
        Row::new(vec!["ACCURACY", acc.as_str()]).style(normal_style),
        Row::new(vec!["GEMS LOST", gems_lost.as_str()]).style(normal_style),
        Row::new(vec!["COLLECTION ACC", colection_accuracy.as_str()]).style(normal_style),
        Row::new(vec![""]),
    ];

    rows.extend(splits);

    let t = Table::new(rows)
    .block(Block::default().borders(Borders::ALL).title("Game Data"))
    .widths(&[
        Constraint::Percentage(25),
        Constraint::Length(40),
        Constraint::Max(10),
    ])
    .header(
        Row::new(header)
            .style(Style::default().fg(Color::Yellow))
            .bottom_margin(0),
    )
    .style(Style::default().fg(Color::Red).bg(Color::Black))
    .column_spacing(1);
    f.render_widget(t, area);
}

pub fn draw_logs<B>(f: &mut Frame<B>, area: Rect, logs: &Vec<String>)
where
    B: Backend,
{
    let log_size = if logs.len() > ((area.height - 2) as usize) {
        logs.len() + 2 - area.height as usize
    } else {
        0
    };
    let logs: Vec<&str> = logs.iter().skip(log_size).map(|x| x.as_str()).collect();

    let events: Vec<ListItem> = logs
        .iter()
        .enumerate()
        .map(|(i, &message)| {
            let log;
            if !logs.is_empty() && i == logs.len() - 1 {
                log = Spans::from(vec![Span::styled(
                    message,
                    Style::default().fg(Color::Black).bg(Color::White),
                )]);
            } else {
                log = Spans::from(vec![Span::raw(message)]);
            }
            ListItem::new(vec![log])
        })
        .collect();

    let events_list = List::new(events)
        .block(Block::default().borders(Borders::ALL).title("Logs"))
        .start_corner(Corner::TopRight)
        .style(Style::default().fg(Color::White).bg(Color::Black));
    f.render_widget(events_list, area);
}
