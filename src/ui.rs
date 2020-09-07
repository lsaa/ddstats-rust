/*use std::{error::Error, io, sync::mpsc, thread};
use termion::{event::{Key}, input::{TermRead, MouseTerminal}, raw::{RawTerminal, IntoRawMode}, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Corner, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Row, Table, Paragraph, Wrap},
    Terminal,
};
use io::Stdout;

static stdout: termion::raw::RawTerminal<std::io::Stdout> = io::stdout().into_raw_mode().unwrap();
static backend: TermionBackend<RawTerminal<Stdout>> = TermionBackend::new(stdout);
static mut terminal : Terminal<TermionBackend<RawTerminal<Stdout>>> = Terminal::new(backend).unwrap();

pub enum Event<I> {
    Input(I),
    Tick,
}

pub fn draw() {
    unsafe {
        let _ = terminal.draw(|f| {
            let selected_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
            let normal_style = Style::default().fg(Color::White);
    
            let header = ["Header1", "Header2", "Header3"];
            let r1 = vec!["OH", "IM", "GAMING"];
            let r2 = vec!["OH", "IM", "GAMING"];
            let items = vec![r1, r2];
            let rows = items.iter().enumerate().map(|(i, item)| {
                if i == 0 {
                    Row::StyledData(item.iter(), selected_style)
                } else {
                    Row::StyledData(item.iter(), normal_style)
                }
            });
    
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(30),
                        Constraint::Percentage(70),
                        //Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(f.size());
    
            let t = Table::new(header.iter(), rows)
                .block(Block::default().borders(Borders::ALL).title("Table"))
                .widths(&[
                    Constraint::Percentage(20),
                    Constraint::Length(20),
                    Constraint::Max(10),
                ])
                .header_style(Style::default().fg(Color::Yellow))
                .style(Style::default().fg(Color::Red).bg(Color::Black))
                .column_spacing(1);
            f.render_widget(t, chunks[1]);
    
    
            let logo: Vec<&str> = crate::consts::LOGO.lines().collect();
            let logo: Vec<Spans> = logo.into_iter()
                .map(|string| 
                    Spans::from(vec![
                        Span::styled(string ,
                        Style::default().add_modifier(Modifier::ITALIC))
                    ])
            ).collect();
            
    
            let paragraph = Paragraph::new(logo)
                    .block(Block::default().borders(Borders::NONE))
                    .style(Style::default().fg(Color::White).bg(Color::Red))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });
    
            f.render_widget(paragraph, chunks[0]);
        });
    }
}
*/