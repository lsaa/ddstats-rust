//
// Funny UI
//

use std::{io::{stdout, Stdout}, time::{Duration, Instant}};
use ddcore_rs::models::StatsBlockWithFrames;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Corner, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Table, Row},
    Frame, Terminal,
};
use serde::Deserialize;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use crate::{client::ConnectionState, config::{self, LogoStyle}, consts::*, threads::{AAS, State, Message}};

use self::{orb_animation::LeviRipple, ascii_canvas::AsciiCanvas};

// Re-exports
pub mod orb_animation;
pub mod ascii_canvas;
pub mod modules;

#[derive(Deserialize, Clone, serde::Serialize)]
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
    draw_ui: bool,
    help: bool,
}

impl UiThread {
    pub async fn init(state: AAS<State>) {
        let mut term = create_term();
        term.clear().expect("Couldn't clear terminal");
        let mut interval = tokio::time::interval(Duration::from_secs_f32(1. / 12.));
        let mut log_list = vec![];
        tokio::spawn(async move {
            let mut extra_settings = ExtraSettings {
                homing_always_visible: crate::config::cfg().ui_conf.always_show_splits,
                draw_ui: crate::config::cfg().ui_conf.enabled,
                help: false
            };

            let (tx, rx) = std::sync::mpsc::channel();
            let _input_handle = {
                let tx = tx.clone();
                std::thread::spawn(move || loop {
                    if event::poll(Duration::from_secs_f32(1. / 10.)).unwrap() {
                        if let CEvent::Key(key) = event::read().unwrap() {
                            tx.send(Event::Input(key)).unwrap();
                        }
                    }
                })
            };

            let mut msg_bus = state.load().msg_bus.0.subscribe();

            loop {
                let state = state.load();
                let cfg = config::cfg();

                tokio::select! {
                    msg = msg_bus.recv() => match msg {
                        Ok(Message::ShowWindow) => { extra_settings.draw_ui = cfg.ui_conf.enabled; },
                        Ok(Message::HideWindow) => { extra_settings.draw_ui = false; let _ = term.clear(); },
                        Ok(Message::Log(data)) => { 
                            log::info!("LOG: {:?}", data);
                            log_list.push(data); 
                        },
                        Ok(Message::Exit) => {
                            disable_raw_mode().expect("I can't");
                            execute!(
                                term.backend_mut(),
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )
                            .expect("FUN");
                            term.show_cursor().expect("NOO");
                            break;
                        }
                        _ => {},
                    },
                    _elapsed = interval.tick() => {
                        let ev_res = rx.try_recv();
                        if ev_res.is_ok() {
                            let ev = ev_res.unwrap();
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
                                        let _ = state.msg_bus.0.send(Message::Exit);
                                        break;
                                    },
                                    KeyCode::F(3) => {
                                        extra_settings.draw_ui = !extra_settings.draw_ui;
                                        let _ = term.clear();
                                    },
                                    KeyCode::F(5) => {
                                        extra_settings.homing_always_visible = !extra_settings.homing_always_visible;
                                    },
                                    KeyCode::F(4) => {
                                        extra_settings.help= !extra_settings.help;
                                    },
                                    KeyCode::F(2) => {
                                        let _ = state.msg_bus.0.send(Message::HideWindow);
                                    },
                                    KeyCode::F(1) => {
                                        let _ = state.msg_bus.0.send(Message::Log("Uploading Replay...".to_string()));
                                        let _ = state.msg_bus.0.send(Message::UploadReplayBuffer);
                                    },
                                    _ => {}
                                },
                            }
                        }
        
                        if !extra_settings.draw_ui {
                            continue;
                        }

                        let read_data = &state.last_poll;
                        let connection_status = &state.conn;
        
                        term.draw(|f| {
                            let mut layout = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints([Constraint::Percentage(100)])
                                .split(f.size());

                            if **connection_status != ConnectionState::Connected
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
                                    .constraints([Constraint::Min(21), Constraint::Percentage(100)])
                                    .horizontal_margin(0)
                                    .vertical_margin(0)
                                    .split(layout[layout.len() - 1]);
        
                                crate::ui::draw_logs(f, info[0], &log_list);
                            }
        
                            if extra_settings.help {
                                crate::ui::draw_help_screen(
                                    f,
                                    info[info.len() - 1],
                                    read_data,
                                    &extra_settings,
                                );
                            } else {
                                crate::ui::draw_info_table(
                                    f,
                                    info[info.len() - 1],
                                    read_data,
                                    &extra_settings,
                                );

                                // OVERDRAW HELP MESSAGE
                                if cfg.ui_conf.show_help_on_border {
                                    f.render_widget(ascii_canvas::BorderOverdraw::new(
                                            cfg.ui_conf.theming.styles.game_data_title, 
                                            cfg.ui_conf.theming.styles.game_data), 
                                        info[info.len() - 1]
                                    );
                                }
                            }
                            
                        })
                        .unwrap();
                    }
                }
            }
        });
    }
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
        ms_count: 0,
    };

    f.render_widget(levi, area);
}

pub fn draw_logo<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let cfg = config::cfg();

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

    let ascii_canvas = AsciiCanvas::new(&logo, Alignment::Center, cfg.ui_conf.theming.styles.logo);
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
    let mut rows = vec![Row::new(vec!["", ""])];
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
        .block(Block::default().borders(Borders::ALL).title(Span::styled("Game Data", cfg.ui_conf.theming.styles.game_data_title)))
        .style(cfg.ui_conf.theming.styles.game_data)
        .column_spacing(1);
    f.render_widget(t, area);
}

pub fn draw_help_screen<B>(
    f: &mut Frame<B>,
    area: Rect,
    _last_data: &StatsBlockWithFrames,
    _extra: &ExtraSettings,
) where
    B: Backend,
{
    let cfg = config::cfg();
    let mut rows = vec![Row::new(vec!["", ""])];

    rows.push(Row::new(vec!["   Manual Replay Upload", "F1"]));
    rows.push(Row::new(vec!["   Send to Tray (Windows)", "F2"]));
    rows.push(Row::new(vec!["   Toggle UI Rendering", "F3"]));
    rows.push(Row::new(vec!["   Toggle Help Screen", "F4"]));
    rows.push(Row::new(vec!["   Show All Splits", "F5"]));
    rows.push(Row::new(vec!["   Quit Safely", "q"]));

    rows.push(Row::new(vec!["", ""]));

    rows.push(Row::new(vec!["   ddstats-go and ddstats website", "VHS (github.com/alexwilkerson)"]));
    rows.push(Row::new(vec!["   Programming / Replay Server", "KyoZM (github.com/lsaa)"]));
    rows.push(Row::new(vec!["   DDInfo API / DDCL", "xvlv (github.com/NoahStolk)"]));
    rows.push(Row::new(vec!["   EXE / Tray Icons", "matt (fuck yuo)"]));
    rows.push(Row::new(vec!["   Special Thanks", "m4ttbush"]));

    let widths = [
        Constraint::Percentage(50),
        Constraint::Length(40),
        Constraint::Max(10),
    ];

    let t = Table::new(rows)
        .block(Block::default().borders(Borders::empty()))
        .widths(&widths)
        .block(Block::default().borders(Borders::ALL).title(Span::styled("HELP", cfg.ui_conf.theming.styles.game_data_title)))
        .style(cfg.ui_conf.theming.styles.game_data)
        .column_spacing(1);
    f.render_widget(t, area);
}

pub fn draw_logs<B>(f: &mut Frame<B>, area: Rect, logs: &[String])
where
    B: Backend,
{
    let cfg = config::cfg();
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
            let log = if !logs.is_empty() && i == logs.len() - 1 {
                Spans::from(vec![Span::styled(
                    message,
                    cfg.ui_conf.theming.styles.most_recent_log,
                )])
            } else {
                Spans::from(vec![Span::styled(message, cfg.ui_conf.theming.styles.log_text)])
            };
            ListItem::new(vec![log])
        })
        .collect();

    let events_list = List::new(events)
        .block(Block::default().borders(Borders::ALL).title(Span::styled("Logs", cfg.ui_conf.theming.styles.logs_title)))
        .start_corner(Corner::TopRight)
        .style(cfg.ui_conf.theming.styles.logs);
    f.render_widget(events_list, area);
}
