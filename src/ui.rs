use tui::backend::Backend;
use std::{io, sync::mpsc, thread};
use termion::{event::{Key}, input::{TermRead, MouseTerminal}, raw::{RawTerminal, IntoRawMode}, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Corner, Direction, Layout, Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Row, Table, Paragraph, Wrap},
    Terminal, Frame,
};
use io::Stdout;
use mpsc::{Receiver, Sender};
use crate::structs::AppDataExtraction;

pub static mut TERMINAL : Option<TermHandler> = None;
static mut CHANNEL : Option<(Sender<Event<Key>>, Receiver<Event<Key>>)> = None;

pub enum Event<I> {
    Input(I),
    Tick,
}

pub fn setup() {
    let stdout = io::stdout().into_raw_mode().unwrap();
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    unsafe { 
        TERMINAL = Some(Terminal::new(backend).unwrap());
        CHANNEL = Some(mpsc::channel());

        let _input_handle = {
            let tx = CHANNEL.as_mut().unwrap().0.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            log::error!("{}", err);
                            return;
                        }
                    }
                }
            })
        };
    }
}

pub fn draw(app_data: &AppDataExtraction) {
    unsafe {
        let _ = TERMINAL.as_mut().unwrap().draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(12),
                        Constraint::Percentage(100),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let info = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Min(28),
                        Constraint::Percentage(100),
                    ]
                    .as_ref(),
                )
                .horizontal_margin(0)
                .vertical_margin(0)
                .split(layout[1]);

            draw_logo(f, layout[0]);
            draw_logs(f, info[0], app_data);
            draw_info_table(f, info[1], app_data);
        
            //Input Handling
            match CHANNEL.as_mut().unwrap().1.try_recv() {
                Ok(data) => match data {
                    Event::Input(data) => match data {
                        Key::Char('q') => std::process::exit(0),
                        Key::Esc => std::process::exit(0),
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            }
        });
    }
}

fn draw_logo<B>(f: &mut Frame<B>, area: Rect) where B: Backend {
    let logo: Vec<&str> = crate::consts::LOGO.lines().collect();
    let logo: Vec<Spans> = logo.into_iter()
        .map(|string| 
            Spans::from(vec![
                Span::styled(string ,
                Style::default().add_modifier(Modifier::BOLD))
            ])
    ).collect();

    let paragraph = Paragraph::new(logo)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White).bg(Color::LightRed))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn draw_logs<B>(f: &mut Frame<B>, area: Rect, app_data: &AppDataExtraction) where B: Backend {
    let log_size = if app_data.logs.len() > ((area.height - 2) as usize)
            {app_data.logs.len() + 2 - area.height as usize} else { 0 };
    let logs : Vec<&str> = app_data.logs.iter().skip(log_size).map(|x| x.as_str()).collect();

    let events: Vec<ListItem> = logs
        .iter()
        .enumerate()
        .map(|(i, &message)| {
            let log;
            if !logs.is_empty() && i == logs.len() - 1 {
                log = Spans::from(vec![Span::styled(message, Style::default().fg(Color::White).bg(Color::DarkGray))]);
            } else {
                log = Spans::from(vec![Span::raw(message)]);
            }
            ListItem::new(vec![
                log,
            ])
        })
        .collect();

    let events_list = List::new(events)
        .block(Block::default().borders(Borders::ALL).title("Logs"))
        .start_corner(Corner::TopRight)
        .style(Style::default().fg(Color::White).bg(Color::Black));
    f.render_widget(events_list, area);
}

fn draw_info_table<B>(f: &mut Frame<B>, area: Rect, app_data: &AppDataExtraction) where B: Backend {
    //let selected_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);

    let user_data = format!("{} ({:.4}s)", app_data.player_name, app_data.pb);
    let timer = format!("{}s", app_data.sio_variables.timer);
    let stt = format!("{:?}", app_data.state);
    let gems = format!("{}", app_data.sio_variables.total_gems);
    let kills = format!("{}", app_data.sio_variables.enemies_killed);
    let acc = format!("{:.4}", app_data.accuracy);
    let homing = format!("{} [MAX {} at {:.4}]", app_data.sio_variables.homing, app_data.homing_max, app_data.homing_max_time);

    let header = [stt.as_str(), user_data.as_str()];
    let r1 = vec!["TIMER", timer.as_str()];
    let r2 = vec!["GEMS", gems.as_str()];
    let r3 = vec!["HOMING", homing.as_str()];
    let r4 = vec!["KILLS", kills.as_str()];
    let r5 = vec!["ACCURACY", acc.as_str()];

    let items = vec![r1, r2, r3, r4, r5];
    let rows = items.iter().enumerate().map(|(i, item)| {
        if i == 0 {
            Row::StyledData(item.iter(), normal_style)
        } else {
            Row::StyledData(item.iter(), normal_style)
        }
    });

    let t = Table::new(header.iter(), rows)
        .block(Block::default().borders(Borders::ALL).title("Game Data"))
        .widths(&[
            Constraint::Percentage(25),
            Constraint::Length(40),
            Constraint::Max(10),
        ])
        .header_gap(0)
        .header_style(Style::default().fg(Color::Yellow))
        .style(Style::default().fg(Color::Red).bg(Color::Black))
        .column_spacing(1);
    f.render_widget(t, area);
}

type TermHandler = Terminal<TermionBackend<termion::screen::AlternateScreen<termion::input::MouseTerminal<RawTerminal<Stdout>>>>>;