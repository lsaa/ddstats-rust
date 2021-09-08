//
// Funny UI
//

use std::io::{stdout, Stdout};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Corner, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame, Terminal,
};

use serde::Deserialize;

use crossterm::{event::EnableMouseCapture, execute, terminal::enable_raw_mode};

use crate::{config, mem::StatsBlockWithFrames};

pub fn create_term() -> Terminal<CrosstermBackend<Stdout>> {
    enable_raw_mode().expect("Couldn't set terminal to raw mode");

    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture).expect("Funny Terminal Business");

    let backend = CrosstermBackend::new(stdout);

    Terminal::new(backend).expect("Couldn't create terminal")
}

pub fn draw_logo<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let cfg = config::CONFIG.with(|c| c.clone());
    let logo: Vec<&str> = cfg.ui_conf.logo.lines().collect();
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
    let cfg = config::CONFIG.with(|z| z.clone());
    let mut rows = vec![];
    for module in &cfg.ui_conf.game_data_modules {
        rows.extend(module.to_rows(last_data));
    }

    let t = Table::new(rows)
        .block(Block::default().borders(Borders::ALL).title("Game Data"))
        .widths(&[
            Constraint::Percentage(25),
            Constraint::Length(40),
            Constraint::Max(10),
        ])
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

// Modular Game Data

#[derive(Deserialize)]
pub enum GameDataModules {
    RunData,
    Timer,
    Gems,
    Homing(bool), // bool: show max homing and time
    Kills,
    Accuracy,
    GemsLost(bool), // bool: show the ways you lost the gems
    CollectionAccuracy,
    HomingSplits(Vec<f32>), // Vec<f32>: split times
    Spacing,
}

#[rustfmt::skip]
impl GameDataModules {
    pub fn to_rows(&self, data: &StatsBlockWithFrames) -> Vec<Row> {
        match self {
            GameDataModules::RunData => create_run_data_rows(&data),
            GameDataModules::Timer => create_timer_rows(&data),
            GameDataModules::Gems => create_gems_rows(&data),
            GameDataModules::Homing(show_max) => create_homing_rows(&data, show_max.clone()),
            GameDataModules::Kills => creake_kills_row(&data),
            GameDataModules::Accuracy => create_accuracy_rows(&data),
            GameDataModules::GemsLost(show_detail) => create_gems_lost_rows(&data, show_detail.clone()),
            GameDataModules::CollectionAccuracy => create_collection_accuracy_rows(&data),
            GameDataModules::HomingSplits(times) => create_homing_splits_rows(&data, times.clone()),
            GameDataModules::Spacing | _ => vec![Row::new([""])],
        }
    }
}

fn create_run_data_rows(data: &&StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    let player = data.block.player_username().to_owned();
    vec![Row::new(["REPLAY".to_string(), player]).style(normal_style)]
}

fn create_timer_rows(data: &&StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    vec![Row::new(["TIMER".into(), format!("{:.4}s", data.block.time_max)]).style(normal_style)]
}

fn create_gems_rows(data: &&StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    vec![Row::new([
        "GEMS".into(),
        format!("{}", data.block.gems_collected),
    ])]
}

fn create_homing_rows(data: &&StatsBlockWithFrames, show_max: bool) -> Vec<Row> {
    let homing_with_max = Row::new([
        "HOMING".into(),
        format!(
            "{} [MAX {} at {:.4}s]",
            data.block.homing, data.block.max_homing, data.block.time_max_homing
        ),
    ]);
    let homing_without_max = Row::new(["HOMING".into(), format!("{}", data.block.homing)]);
    if show_max {
        vec![homing_with_max]
    } else {
        vec![homing_without_max]
    }
}

fn creake_kills_row(data: &&StatsBlockWithFrames) -> Vec<Row> {
    vec![Row::new(["KILLS".into(), format!("{}", data.block.kills)])]
}

fn create_accuracy_rows(data: &&StatsBlockWithFrames) -> Vec<Row> {
    if let Some(frame) = data.frames.last() {
        let mut acc = frame.daggers_hit as f32 / frame.daggers_fired as f32;
        if acc.is_nan() {
            acc = 100.00;
        }
        return vec![Row::new(["ACCURACY".into(), format!("{:.2}%", acc)])];
    }
    vec![Row::new(["ACCURACY", "100.00%"])]
}

#[rustfmt::skip]
fn create_gems_lost_rows(data: &&StatsBlockWithFrames, details: bool) -> Vec<Row> {
    let total_gems_lost = data.block.gems_eaten + data.block.gems_despawned;
    let gems_lost_detail = format!(
        "{} [{} DESPAWNED; {} EATEN]",
        total_gems_lost, data.block.gems_despawned, data.block.gems_eaten
    );
    let gems_lost_min = format!("{}", total_gems_lost);
    vec![Row::new(["GEMS LOST".into(), if details { gems_lost_detail } else { gems_lost_min }])]
}

fn create_collection_accuracy_rows(data: &&StatsBlockWithFrames) -> Vec<Row> {
    if let Some(frame) = data.frames.last() {
        let total_gems_lost = data.block.gems_eaten + data.block.gems_despawned;
        let acc = (data.block.gems_total - total_gems_lost) as f32 / data.block.gems_total as f32;
        let mut acc = frame.daggers_hit as f32 / frame.daggers_fired as f32;
        if acc.is_nan() {
            acc = 100.00;
        }
        return vec![Row::new(["COLLECTION ACC".into(), format!("{:.2}%", acc)])];
    }
    vec![Row::new(["COLLECTION ACC", "100.00%"])]
}

fn create_homing_splits_rows(data: &&StatsBlockWithFrames, times: Vec<f32>) -> Vec<Row> {
    let mut splits = Vec::new();
    let normal_style = Style::default().fg(Color::White);
    let real_timer = data.block.time_max + data.block.starting_time;
    for time in times {
        if time < real_timer {
            if let Some(time_frame) = data.get_frame_for_time(time) {
                splits.push(
                    Row::new(vec![
                        "SPLIT".to_owned(),
                        format!("{} HOMING AT {:.1}s", time_frame.homing, time),
                    ])
                    .style(normal_style),
                );
            }
        }
    }
    splits
}
