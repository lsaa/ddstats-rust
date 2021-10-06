//
// Funny UI
//

use std::{
    io::{stdout, Stdout},
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};

use num_traits::FromPrimitive;
use regex::Regex;
use tokio::sync::RwLock;
use tui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Alignment, Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Row, Table, Widget},
    Frame, Terminal,
};

use serde::Deserialize;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};

use crate::{client::{ConnectionState, GameStatus}, config::{self, LogoStyle}, consts::*, mem::StatsBlockWithFrames};

thread_local! {
    static LEVI: Arc<LeviRipple> = Arc::new(LeviRipple { start_time: Instant::now() })
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
    DdclOutOfDateWarning,
    Spacing,
}

#[derive(Deserialize, Clone)]
pub enum SizeStyle {
    Minimal,
    Compact,
    Full,
}

pub enum Event<I> {
    Input(I),
}

pub struct UiThread;

#[derive(Clone, Debug)]
pub struct ExtraSettings {
    homing_always_visible: bool,
}

impl UiThread {
    pub async fn init(
        latest_data: Arc<RwLock<StatsBlockWithFrames>>,
        logs: Arc<RwLock<Vec<String>>>,
        connected: Arc<RwLock<ConnectionState>>,
        exit_broadcast: tokio::sync::broadcast::Sender<bool>,
        color_edit_styles: Arc<RwLock<crate::config::Styles>>,
    ) {
        let mut term = create_term();
        term.clear().expect("Couldn't clear terminal");
        let cfg = config::cfg();
        let mut interval = tokio::time::interval(Duration::from_secs_f32(1. / 20.));
        tokio::spawn(async move {
            let mut in_color_mode = false;
            let mut extra_settings = ExtraSettings {
                homing_always_visible: false,
            };

            let (tx, rx) = mpsc::channel();
            let _input_handle = {
                let tx = tx.clone();
                std::thread::spawn(move || loop {
                    if event::poll(Duration::from_secs_f32(1. / 20.)).unwrap() {
                        if let CEvent::Key(key) = event::read().unwrap() {
                            tx.send(Event::Input(key)).unwrap();
                        }
                    }
                })
            };

            loop {
                interval.tick().await;

                if let Ok(ev) = rx.try_recv() {
                    match ev {
                        Event::Input(event) => match event.code {
                            KeyCode::Char('q') => {
                                disable_raw_mode().expect("I can't");
                                execute!(
                                    term.backend_mut(),
                                    LeaveAlternateScreen,
                                    DisableMouseCapture
                                )
                                .expect("FUN");
                                term.show_cursor().expect("NOO");
                                exit_broadcast
                                    .send(true)
                                    .expect("Coudln't send exit broadcast");
                                break;
                            }
                            KeyCode::F(3) => {
                                in_color_mode = !in_color_mode;
                            },
                            KeyCode::F(5) => {
                                extra_settings.homing_always_visible = !extra_settings.homing_always_visible;
                            }
                            _ => {}
                        },
                    }
                }
                let read_data = latest_data.read().await;
                let log_list = logs.read().await;
                let connection_status = connected.read().await;

                if in_color_mode {
                    let s = color_edit_styles.read().await;
                    draw_color_editor_mode(&mut term, &*s);
                    continue;
                }

                term.draw(|f| {
                    let mut layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(100)])
                        .split(f.size());

                    if *connection_status != ConnectionState::Connected
                        && cfg.ui_conf.orb_connection_animation
                    {
                        draw_levi(f, layout[0]);
                        return;
                    }

                    if cfg.ui_conf.logo_style != LogoStyle::Off {
                        let max_w = LOGO_NEW.lines().fold(
                            LOGO_NEW.lines().next().unwrap().chars().count(),
                            |acc, x| {
                                if x.chars().count() > acc {
                                    x.chars().count()
                                } else {
                                    acc
                                }
                            },
                        );

                        let height = match cfg.ui_conf.logo_style {
                            LogoStyle::Auto => {
                                if layout[0].width as usize >= max_w {
                                    LOGO_NEW.lines().count()
                                } else {
                                    LOGO_MINI.lines().count()
                                }
                            }
                            LogoStyle::Mini => LOGO_MINI.lines().count(),
                            LogoStyle::Full => LOGO_NEW.lines().count(),
                            LogoStyle::Off => 0,
                        };

                        layout = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Min(height as u16 + 1),
                                Constraint::Percentage(100),
                            ])
                            .split(f.size());

                        crate::ui::draw_logo(f, layout[0]);
                    }

                    let mut info = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(100)])
                        .horizontal_margin(0)
                        .vertical_margin(0)
                        .split(layout[layout.len() - 1]);

                    if !cfg.ui_conf.hide_logs {
                        info = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Min(20), Constraint::Percentage(100)])
                            .horizontal_margin(0)
                            .vertical_margin(0)
                            .split(layout[layout.len() - 1]);

                        crate::ui::draw_logs(f, info[0], &log_list);
                    }

                    crate::ui::draw_info_table(
                        f,
                        info[info.len() - 1],
                        &read_data,
                        &extra_settings,
                    );
                })
                .unwrap();
            }
        });
    }
}

fn draw_color_editor_mode(
    term: &mut Terminal<CrosstermBackend<Stdout>>,
    styles: &crate::config::Styles,
) {
    let cfg = crate::config::cfg();
    term.draw(|f| {
        let mut layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(f.size());

        if cfg.ui_conf.logo_style != LogoStyle::Off {
            let max_w = LOGO_NEW.lines().fold(
                LOGO_NEW.lines().next().unwrap().chars().count(),
                |acc, x| {
                    if x.chars().count() > acc {
                        x.chars().count()
                    } else {
                        acc
                    }
                },
            );

            let height = match cfg.ui_conf.logo_style {
                LogoStyle::Auto => {
                    if layout[0].width as usize >= max_w {
                        LOGO_NEW.lines().count()
                    } else {
                        LOGO_MINI.lines().count()
                    }
                }
                LogoStyle::Mini => LOGO_MINI.lines().count(),
                LogoStyle::Full => LOGO_NEW.lines().count(),
                LogoStyle::Off => 0,
            };

            layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(height as u16 + 1),
                    Constraint::Percentage(100),
                ])
                .split(f.size());

            crate::ui::draw_logo_color_editor(f, layout[0], &styles);
        }

        let mut info = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .horizontal_margin(0)
            .vertical_margin(0)
            .split(layout[layout.len() - 1]);

        if !cfg.ui_conf.hide_logs {
            info = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(20), Constraint::Percentage(100)])
                .horizontal_margin(0)
                .vertical_margin(0)
                .split(layout[layout.len() - 1]);

            crate::ui::draw_logs_color_edit(
                f,
                info[0],
                &vec!["First".into(), "Second".into()],
                &styles,
            );
        }

        crate::ui::draw_info_table_color_edit(f, info[info.len() - 1], &styles);
    })
    .unwrap();
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

    let max_w = LOGO_NEW.lines().fold(
        LOGO_NEW.lines().next().unwrap().chars().count(),
        |acc, x| {
            if x.chars().count() > acc {
                x.chars().count()
            } else {
                acc
            }
        },
    );

    let logo = match cfg.ui_conf.logo_style {
        config::LogoStyle::Off => "".to_string(),
        config::LogoStyle::Auto => {
            if area.width >= max_w as u16 {
                LOGO_NEW.to_string()
            } else {
                LOGO_MINI.to_string()
            }
        }
        config::LogoStyle::Mini => LOGO_MINI.to_string(),
        config::LogoStyle::Full => LOGO_NEW.to_string(),
    };

    let ascii_canvas = AsciiCanvas::new(&logo, Alignment::Center, cfg.ui_conf.style.logo);
    f.render_widget(ascii_canvas, area);
}

pub fn draw_logo_color_editor<B>(f: &mut Frame<B>, area: Rect, styles: &crate::config::Styles)
where
    B: Backend,
{
    let cfg = config::CONFIG.with(|c| c.clone());

    let max_w = LOGO_NEW.lines().fold(
        LOGO_NEW.lines().next().unwrap().chars().count(),
        |acc, x| {
            if x.chars().count() > acc {
                x.chars().count()
            } else {
                acc
            }
        },
    );

    let logo = match cfg.ui_conf.logo_style {
        config::LogoStyle::Off => "".to_string(),
        config::LogoStyle::Auto => {
            if area.width >= max_w as u16 {
                LOGO_NEW.to_string()
            } else {
                LOGO_MINI.to_string()
            }
        }
        config::LogoStyle::Mini => LOGO_MINI.to_string(),
        config::LogoStyle::Full => LOGO_NEW.to_string(),
    };

    let ascii_canvas = AsciiCanvas::new(&logo, Alignment::Center, styles.logo);
    f.render_widget(ascii_canvas, area);
}

pub fn draw_info_table<B>(
    f: &mut Frame<B>,
    area: Rect,
    last_data: &StatsBlockWithFrames,
    extra: &ExtraSettings,
) where
    B: Backend,
{
    let cfg = config::cfg();
    let mut rows = vec![];
    let colorizer = GameDataColorizer { styles: None };
    for module in &cfg.ui_conf.game_data_modules {
        rows.extend(module.to_rows(last_data, extra));
    }

    let dist = cfg.ui_conf.column_distance;
    let widths = [
        Constraint::Percentage(dist),
        Constraint::Length(40),
        Constraint::Max(10),
    ];

    let t = Table::new(rows)
        .block(Block::default().borders(Borders::empty()))
        .widths(&widths)
        .block(Block::default().borders(Borders::ALL).title("Game Data"))
        .style(cfg.ui_conf.style.game_data)
        .column_spacing(1);
    f.render_widget(t, area);
    f.render_widget(colorizer, area);
}

pub fn draw_info_table_color_edit<B>(f: &mut Frame<B>, area: Rect, styles: &crate::config::Styles)
where
    B: Backend,
{
    let cfg = config::cfg();
    let mut rows = vec![];
    let colorizer = GameDataColorizer {
        styles: Some(styles.clone()),
    };
    let normal_style = Style::default().fg(Color::White);

    rows.push(
        Row::new(vec![
            "REGULAR TEXT".into(),
            "F3 TO EXIT COLOR EDITOR".to_string(),
        ])
        .style(normal_style),
    );
    rows.push(Row::new(vec!["SPLIT".into(), " Levi: 123 (+22)".to_string()]).style(normal_style));
    rows.push(Row::new(vec!["SPLIT".into(), "Games: 444 (-20)".to_string()]).style(normal_style));

    let dist = cfg.ui_conf.column_distance;
    let widths = [
        Constraint::Percentage(dist),
        Constraint::Length(40),
        Constraint::Max(10),
    ];

    let t = Table::new(rows)
        .block(Block::default().borders(Borders::ALL).title("Game Data"))
        .widths(&widths)
        .style(styles.game_data)
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

pub fn draw_logs_color_edit<B>(
    f: &mut Frame<B>,
    area: Rect,
    logs: &Vec<String>,
    styles: &crate::config::Styles,
) where
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
                log = Spans::from(vec![Span::styled(message, styles.most_recent_log)]);
            } else {
                log = Spans::from(vec![Span::styled(message, styles.log_text)]);
            }
            ListItem::new(vec![log])
        })
        .collect();

    let events_list = List::new(events)
        .block(Block::default().borders(Borders::ALL).title("Logs"))
        .start_corner(Corner::TopRight)
        .style(styles.logs);
    f.render_widget(events_list, area);
}

#[allow(unreachable_patterns)] #[rustfmt::skip]
impl<'a> GameDataModules {
    pub fn to_rows(&'a self, data: &'a StatsBlockWithFrames, extra: &'a ExtraSettings) -> Vec<Row> {
        match self {
            GameDataModules::RunData => create_run_data_rows(&data),
            GameDataModules::Timer => create_timer_rows(&data),
            GameDataModules::Gems => create_gems_rows(&data),
            GameDataModules::Homing(size_style) => create_homing_rows(&data, size_style.clone()),
            GameDataModules::Kills => creake_kills_row(&data),
            GameDataModules::Accuracy => create_accuracy_rows(&data),
            GameDataModules::GemsLost(size_style) => create_gems_lost_rows(&data, size_style.clone()),
            GameDataModules::CollectionAccuracy => create_collection_accuracy_rows(&data),
            GameDataModules::HomingSplits(times) => create_homing_splits_rows(&data, times.clone(), extra.clone()),
            GameDataModules::HomingUsed => create_homing_used_rows(&data),
            GameDataModules::DaggersEaten => create_daggers_eaten_rows(&data),
            GameDataModules::DdclOutOfDateWarning => ddcl_warning_rows(&data),
            GameDataModules::Spacing | _ => vec![Row::new([""])],
        }
    }
}

fn create_run_data_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    let player = data.block.replay_player_username().to_owned();
    let status = FromPrimitive::from_i32(data.block.status);
    let status = match status {
        Some(st) => match st {
            GameStatus::Menu => "MENU",
            GameStatus::Lobby => "LOBBY",
            GameStatus::Dead => crate::consts::DEATH_TYPES[data.block.death_type as usize],
            GameStatus::Title => "TITLE",
            GameStatus::Playing => "ALIVE",
            GameStatus::OtherReplay | GameStatus::OwnReplayFromLastRun | GameStatus::OwnReplayFromLeaderboard => "REPLAY",
        }.to_string(),
        None => "CONNECTING".to_string()
    };
    vec![Row::new([status, player]).style(normal_style)]
}

fn create_timer_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);

    if data.block.is_replay {
        return vec![Row::new([
            "TIMER".into(),
            format!("{:.4}/{:.4}s", data.block.time, data.block.time_max + data.block.starting_time),
        ]).style(normal_style)];
    }

    vec![Row::new([
        "TIMER".into(),
        format!("{:.4}s", data.block.time_max + data.block.starting_time),
    ]).style(normal_style)]
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

#[allow(unreachable_patterns)]
fn create_homing_rows(data: &StatsBlockWithFrames, style: SizeStyle) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    let time = if data.block.time_lvl3 == 0. { 0. } else { data.block.time_max_homing };
    match style {
        SizeStyle::Full => {
            vec![Row::new([
                "HOMING".into(),
                format!(
                    "{} [MAX {} at {:.4}s]",
                    data.block.homing, data.block.max_homing, time
                ),
            ])
            .style(normal_style)]
        }
        SizeStyle::Compact => {
            vec![Row::new([
                "HOMING".into(),
                format!(
                    "{} [{} @ {:.4}s]",
                    data.block.homing, data.block.max_homing, time
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
        let pacifist = frame.daggers_hit == 0;
        let pacifist = if pacifist { "[PACIFIST]" } else { "" };
        if acc.is_nan() {
            acc = 1.00;
        }
        return vec![
            Row::new(["ACCURACY".into(), format!("{:.2}% {}", acc * 100., pacifist)]).style(normal_style)
        ];
    }
    vec![Row::new(["ACCURACY", "100.00% [PACIFIST]"]).style(normal_style)]
}

#[rustfmt::skip] #[allow(unreachable_patterns)]
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
            let compact = format!("{} [{} / {}]", total_gems_lost, data.block.gems_despawned, data.block.gems_eaten);
            vec![Row::new(["GEMS LOST".into(), compact]).style(normal_style)]
        },
        SizeStyle::Minimal => {
            vec![Row::new(["GEMS LOST".into(), gems_lost_min]).style(normal_style)]
        },
        _ => vec![]
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

fn create_homing_splits_rows(
    data: &StatsBlockWithFrames,
    times: Vec<(String, f32)>,
    extra: ExtraSettings,
) -> Vec<Row> {
    let mut splits = Vec::new();
    let normal_style = Style::default().fg(Color::White);
    let real_timer = data.block.time_max + data.block.starting_time;

    if extra.homing_always_visible {
        let mut last_split = 105;
        for (name, time) in times {
            let time_frame = data.get_frame_for_time(time);
            let hom = if time_frame.is_some() { time_frame.unwrap().homing } else { data.block.homing };
            splits.push(
                Row::new(vec![
                    "SPLIT".to_owned(),
                    format!(
                        "{:>4}: {:<4} ({:+})",
                        name,
                        hom,
                        hom - &last_split
                    ),
                ])
                .style(normal_style),
            );
            last_split = hom;
        }
        return splits;
    }

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
        format!("{}", data.homing_usage_from_frames(Some(data.block.time))),
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

fn ddcl_warning_rows(_data: &StatsBlockWithFrames) -> Vec<Row> {
    let normal_style = Style::default().fg(Color::White);
    if *crate::web_clients::dd_info::DDLC_UP_TO_DATE.as_ref() {
        return vec![];
    }
    vec![Row::new([
        String::from("DDCL WARN"),
        "OUT OF DATE CLIENT ||  WON'T SUBMIT".into(),
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

pub struct GameDataColorizer {
    pub styles: Option<crate::config::Styles>,
}

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

        let mut split_name_style = cfg.ui_conf.style.split_name;
        let mut split_pos_style = cfg.ui_conf.style.split_diff_pos;
        let mut split_neg_style = cfg.ui_conf.style.split_diff_neg;
        let mut split_value_style = cfg.ui_conf.style.split_value;

        if self.styles.is_some() {
            let unw = self.styles.unwrap();
            split_value_style = unw.split_value;
            split_neg_style = unw.split_diff_neg;
            split_name_style = unw.split_name;
            split_pos_style = unw.split_diff_pos;
        }

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
                        buf.get_mut(x, y).set_style(split_name_style);
                    }
                    for x in count.range() {
                        let x = x as u16 + area.x - 2;
                        buf.get_mut(x, y).set_style(split_value_style);
                    }

                    let dstyle = if diff.as_str().to_string().contains("+") {
                        split_pos_style
                    } else {
                        split_neg_style
                    };

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
        let time_elapsed = lev.start_time.elapsed().div_f32(200.);
        // Different Messages so it can always be centered
        let msg1 = "Waiting for Devil Daggers";
        let msg2 = "Waiting for Game";
        let mut tmp = [0; 4];
        for y in 0..area.height {
            for x in 0..area.width {
                let map_x = -((area.width as f32 - x as f32) - (area.width as f32 / 2.));
                let map_y = (area.height as f32 - y as f32) - (area.height as f32 / 2.);
                let height = (((map_x * map_x + map_y * map_y).sqrt() / 5.)
                    - time_elapsed.as_millis() as f32)
                    .sin();
                let height = (height * (255. / 2.)) + (255. / 2.);
                let height = height.clamp(20., 255.);
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

pub struct AsciiCanvas {
    lines: Vec<String>,
    alignment: Alignment,
    style: Style,
}

impl AsciiCanvas {
    pub fn new(base: &str, alignment: Alignment, style: Style) -> Self {
        Self {
            lines: base.lines().map(|st| st.to_string()).collect(),
            alignment,
            style,
        }
    }
}

#[rustfmt::skip]
impl<'a> Widget for AsciiCanvas {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let max_w = self.lines.iter().fold(0, |acc, x| if x.chars().count() > acc { x.chars().count() } else { acc });
        let max_h = self.lines.len();
        if area.width < max_h as u16 || area.height < max_h as u16 { return; }

        let left = match self.alignment {
            Alignment::Center => (area.width / 2).saturating_sub(max_w as u16 / 2),
            Alignment::Right => area.width.saturating_sub(max_w as u16),
            Alignment::Left => 0,
        };

        for y in 0..area.height {
            for x in 0..area.width {
                buf.get_mut(area.x + x, area.y + y).set_style(self.style);
            }
        }

        for (y, line) in self.lines.iter().enumerate() {
            for (x, c) in line.chars().enumerate() {
                buf.get_mut(left + x as u16 + area.x, y as u16 + area.y)
                    .set_symbol(c.to_string().as_str())
                    .set_style(self.style);
            }
        }

    }
}
