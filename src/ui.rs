//
// Funny UI
//

use std::{
    io::{stdout, Stdout},
    sync::Arc,
    time::Instant,
};

use regex::Regex;
use tui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Alignment, Constraint, Corner, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Widget, Wrap},
    Frame, Terminal,
};

use serde::Deserialize;

use crossterm::{event::EnableMouseCapture, execute, terminal::enable_raw_mode};

use crate::{config, mem::StatsBlockWithFrames};

thread_local! {
    static LEVI: Arc<LeviRipple> = Arc::new(LeviRipple { start_time: Instant::now() })
}

pub fn create_term() -> Terminal<CrosstermBackend<Stdout>> {
    enable_raw_mode().expect("Couldn't set terminal to raw mode");

    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture).expect("Funny Terminal Business");

    let backend = CrosstermBackend::new(stdout);

    Terminal::new(backend).expect("Couldn't create terminal")
}

pub fn draw_levi<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let levi = LeviRipple {
        start_time: Instant::now(),
    };

    f.render_widget(levi, area);
}

pub fn draw_logo<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let cfg = config::CONFIG.with(|c| c.clone());
    let logo: Vec<&str> = cfg.ui_conf.logo.0.lines().collect();
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
        .style(cfg.ui_conf.style.logo)
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
    let colorizer = GameDataColorizer {};
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
        .style(cfg.ui_conf.style.game_data)
        .column_spacing(1);
    f.render_widget(t, area);
    f.render_widget(colorizer, area);
}

pub fn draw_logs<B>(f: &mut Frame<B>, area: Rect, logs: &Vec<String>)
where
    B: Backend,
{
    let cfg = config::CONFIG.with(|z| z.clone());
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
                    cfg.ui_conf.style.most_recent_log,
                )]);
            } else {
                log = Spans::from(vec![Span::styled(message, cfg.ui_conf.style.log_text)]);
            }
            ListItem::new(vec![log])
        })
        .collect();

    let events_list = List::new(events)
        .block(Block::default().borders(Borders::ALL).title("Logs"))
        .start_corner(Corner::TopRight)
        .style(cfg.ui_conf.style.logs);
    f.render_widget(events_list, area);
}

// Modular Game Data

#[derive(Deserialize)]
pub enum GameDataModules {
    RunData,
    Timer,
    Gems,
    Homing(SizeStyle), // Minimal, Compact, Full
    Kills,
    Accuracy,
    GemsLost(SizeStyle), // Minimal, Compact, Full
    CollectionAccuracy,
    HomingSplits(Vec<(String, f32)>), // Vec<(String, f32)>: split times and names
    HomingUsed,
    DaggersEaten,
    Spacing,
}

#[derive(Deserialize, Clone)]
pub enum SizeStyle {
    Minimal,
    Compact,
    Full,
}

#[allow(unreachable_patterns)] #[rustfmt::skip]
impl<'a> GameDataModules {
    pub fn to_rows(&'a self, data: &'a StatsBlockWithFrames) -> Vec<Row> {
        match self {
            GameDataModules::RunData => create_run_data_rows(&data),
            GameDataModules::Timer => create_timer_rows(&data),
            GameDataModules::Gems => create_gems_rows(&data),
            GameDataModules::Homing(size_style) => create_homing_rows(&data, size_style.clone()),
            GameDataModules::Kills => creake_kills_row(&data),
            GameDataModules::Accuracy => create_accuracy_rows(&data),
            GameDataModules::GemsLost(size_style) => create_gems_lost_rows(&data, size_style.clone()),
            GameDataModules::CollectionAccuracy => create_collection_accuracy_rows(&data),
            GameDataModules::HomingSplits(times) => create_homing_splits_rows(&data, times.clone()),
            GameDataModules::HomingUsed => create_homing_used_rows(&data),
            GameDataModules::DaggersEaten => create_daggers_eaten_rows(&data),
            GameDataModules::Spacing | _ => vec![Row::new([""])],
        }
    }
}

fn create_run_data_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    let player = data.block.replay_player_username().to_owned();
    vec![Row::new(["REPLAY".to_string(), player]).style(normal_style)]
}

fn create_timer_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    vec![Row::new([
        "TIMER".into(),
        format!("{:.4}s", data.block.time_max + data.block.starting_time),
    ])
    .style(normal_style)]
}

fn create_gems_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    vec![Row::new(["GEMS".into(), format!("{}", data.block.gems_collected)]).style(normal_style)]
}

fn get_homing(data: &StatsBlockWithFrames) -> u32 {
    let homing;
    if let Some(most_recent) = data.frames.last() {
        homing = most_recent.homing;
    } else {
        homing = 0;
    }
    homing as u32
}

fn create_homing_rows(data: &StatsBlockWithFrames, style: SizeStyle) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    match style {
        SizeStyle::Full => {
            vec![Row::new([
                "HOMING".into(),
                format!(
                    "{} [MAX {} at {:.4}s]",
                    data.block.homing, data.block.max_homing, data.block.time_max_homing
                ),
            ])
            .style(normal_style)]
        }
        SizeStyle::Compact => {
            vec![Row::new([
                "HOMING".into(),
                format!(
                    "{} [{} @ {:.4}s]",
                    data.block.homing, data.block.max_homing, data.block.time_max_homing
                ),
            ])
            .style(normal_style)]
        }
        SizeStyle::Minimal => {
            vec![Row::new(["HOMING".into(), format!("{}", get_homing(&data))]).style(normal_style)]
        }
        _ => vec![],
    }
}

fn creake_kills_row(data: &StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    vec![Row::new(["KILLS".into(), format!("{}", data.block.kills)]).style(normal_style)]
}

fn create_accuracy_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    if let Some(frame) = data.frames.last() {
        let mut acc = frame.daggers_hit as f32 / frame.daggers_fired as f32;
        if acc.is_nan() {
            acc = 1.00;
        }
        return vec![
            Row::new(["ACCURACY".into(), format!("{:.2}%", acc * 100.)]).style(normal_style)
        ];
    }
    vec![Row::new(["ACCURACY", "100.00%"]).style(normal_style)]
}

#[rustfmt::skip]
fn create_gems_lost_rows(data: &StatsBlockWithFrames, style: SizeStyle) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    let total_gems_lost = data.block.gems_eaten + data.block.gems_despawned;
    let gems_lost_detail = format!(
        "{} [{} DESPAWNED; {} EATEN]",
        total_gems_lost, data.block.gems_despawned, data.block.gems_eaten
    );
    let gems_lost_min = format!("{}", total_gems_lost);

    match style {
        SizeStyle::Full => {
            vec![Row::new(["GEMS LOST".into(), gems_lost_detail]).style(normal_style)]
        },
        SizeStyle::Compact => {
            let compact = format!("{} [{}+{}]", total_gems_lost, data.block.gems_despawned, data.block.gems_eaten);
            vec![Row::new(["GEMS LOST".into(), compact]).style(normal_style)]
        },
        SizeStyle::Minimal => {
            vec![Row::new(["GEMS LOST".into(), gems_lost_min]).style(normal_style)]
        }
    }
}

fn create_collection_accuracy_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    if let Some(frame) = data.frames.last() {
        let total_gems_lost = frame.gems_eaten + frame.gems_despawned;
        let mut acc = (frame.gems_total - total_gems_lost) as f32 / frame.gems_total as f32;
        if acc.is_nan() {
            acc = 1.00;
        }
        return vec![
            Row::new(["COLLECTION ACC".into(), format!("{:.2}%", acc * 100.)]).style(normal_style),
        ];
    }
    vec![Row::new(["COLLECTION ACC", "100.00%"]).style(normal_style)]
}

fn create_homing_splits_rows(data: &StatsBlockWithFrames, times: Vec<(String, f32)>) -> Vec<Row> {
    let mut splits = Vec::new();
    let normal_style = Style::default().fg(Color::White);
    let real_timer = data.block.time_max + data.block.starting_time;
    let mut last_split = 105;
    for (name, time) in times {
        if time < real_timer {
            if let Some(time_frame) = data.get_frame_for_time(time) {
                splits.push(
                    Row::new(vec![
                        "SPLIT".to_owned(),
                        format!(
                            "{:>4}: {:<4} ({:+})",
                            name,
                            time_frame.homing,
                            time_frame.homing - &last_split
                        ),
                    ])
                    .style(normal_style),
                );
                last_split = time_frame.homing;
            }
        }
    }
    splits
}

fn create_homing_used_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    vec![Row::new([
        "HOMING USED".into(),
        format!("{}", data.homing_usage_from_frames()),
    ])
    .style(normal_style)]
}

fn create_daggers_eaten_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    vec![Row::new([
        "DAGGERS EATEN".into(),
        format!("{}", data.block.daggers_eaten),
    ])
    .style(normal_style)]
}

pub struct LeviRipple {
    pub start_time: Instant,
}

const TERM_COLOR_RAMP: &str = " .:-=+*#%@█";

fn char_from_intensity(intensity: u8) -> char {
    let w = (intensity as f32 / 255.).clamp(0., 1.);
    let m = (TERM_COLOR_RAMP.len() - 1) as f32 * w;
    TERM_COLOR_RAMP
        .chars()
        .nth(m.floor().clamp(2., 10.) as usize)
        .unwrap()
}

pub struct GameDataColorizer {}

fn buffer_as_lines(buf: &Buffer, area: &Rect) -> Vec<String> {
    let w = buf.area().width;
    let mut res = vec![];
    let mut current_string = String::new();
    let mut x = 0;
    let mut y = 0;
    buf.content().into_iter().for_each(|cell| {
        if x == w {
            if !current_string.is_empty() {
                res.push(current_string.clone());
            }
            current_string = String::new();
            x = 0;
            y += 1;
        }
        if area.intersects(Rect::new(x, y, 1, 1)) {
            current_string.push_str(cell.symbol.clone().as_str());
        }
        x += 1;
    });
    res
}

impl<'a> Widget for GameDataColorizer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let cfg = config::CONFIG.with(|x| x.clone());
        let re = Regex::new(r"SPLIT\s*(\S*):\s*(\d*)\s*\(([\+\-]?\d*)\)").unwrap();
        let lines = buffer_as_lines(&buf, &area);
        for (y, line) in lines.iter().enumerate() {
            // Color Splits
            if line.contains("SPLIT") {
                for cap in re.captures_iter(&line) {
                    let y = y as u16 + area.y;
                    let (name, count, diff) = (
                        cap.get(1).unwrap(),
                        cap.get(2).unwrap(),
                        cap.get(3).unwrap(),
                    );
                    for x in name.range() {
                        let x = x as u16 + area.x - 2;
                        buf.get_mut(x, y).set_style(cfg.ui_conf.style.split_name);
                    }
                    for x in count.range() {
                        let x = x as u16 + area.x - 2;
                        buf.get_mut(x, y).set_style(cfg.ui_conf.style.split_value);
                    }

                    let dstyle = if diff.as_str().to_string().contains("+") {
                        cfg.ui_conf.style.split_diff_pos
                    } else {
                        cfg.ui_conf.style.split_diff_neg
                    };
                    buf.get_mut(diff.range().start as u16 - 1 + area.x, y)
                        .set_style(dstyle);
                    buf.get_mut(diff.range().end as u16 + 1 + area.x, y)
                        .set_style(dstyle);

                    for x in diff.range() {
                        let x = x as u16 + area.x - 2;
                        buf.get_mut(x, y).set_style(dstyle);
                    }
                    break;
                }
            }
        }
    }
}

impl<'a> Widget for LeviRipple {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lev = LEVI.with(|z| z.clone());
        let time_elapsed = lev.start_time.elapsed().div_f32(50.);
        // Different Messages so it can always be centered
        let msg1 = "Waiting for Devil Daggers";
        let msg2 = "Waiting for Game";
        let mut tmp = [0; 4];
        for y in 0..area.height {
            for x in 0..area.width {
                let map_x = -((area.width as f32 - x as f32) - (area.width as f32 / 2.));
                let map_y = (area.height as f32 - y as f32) - (area.height as f32 / 2.);
                let height = (((map_x * map_x + map_y * map_y).sqrt() / 9.)
                    - time_elapsed.as_millis() as f32)
                    .sin();
                let height = (height * (255. / 2.)) + (255. / 2.);
                let height = height.clamp(90., 255.);
                buf.get_mut(x, y)
                    .set_symbol(char_from_intensity(height as u8).encode_utf8(&mut tmp))
                    .set_style(Style::default().bg(Color::Rgb(0, 0, 0)).fg(Color::Rgb(
                        height as u8,
                        0,
                        0,
                    )));
            }
        }

        let msg;
        if area.width % 2 == 0 {
            msg = msg2;
        } else {
            msg = msg1;
        }

        let mut s = "".to_owned();
        for _ in 0..msg.len() {
            s.push_str("#");
        }

        for _ in 0..16 {
            s.push_str("#");
        }

        buf.set_span(
            area.width / 2 - (msg.len() / 2) as u16 - 8,
            area.height / 2 - 1,
            &Span::styled(
                s.clone(),
                Style::default().bg(Color::Black).fg(Color::Black),
            ),
            msg.len() as u16 + 16,
        );
        buf.set_span(
            area.width / 2 - (msg.len() / 2) as u16 - 8,
            area.height / 2 + 1,
            &Span::styled(
                s.clone(),
                Style::default().bg(Color::Black).fg(Color::Black),
            ),
            msg.len() as u16 + 16,
        );
        buf.set_span(
            area.width / 2 - (msg.len() / 2) as u16 - 8,
            area.height / 2 + 0,
            &Span::styled(
                s.clone(),
                Style::default().bg(Color::Black).fg(Color::Black),
            ),
            msg.len() as u16 + 16,
        );
        buf.set_span(
            area.width / 2 - (msg.len() / 2) as u16,
            area.height / 2,
            &Span::styled(msg, Style::default().bg(Color::Black).fg(Color::White)),
            msg.len() as u16,
        );
    }
}
