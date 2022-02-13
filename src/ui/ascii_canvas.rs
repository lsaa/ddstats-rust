//
//  Ascii Canvas widget for tui-rs
//

use tui::{layout::{Alignment, Rect}, style::Style, widgets::Widget, buffer::Buffer, text::Span};

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


pub struct BorderOverdraw {
    style: Style,
}

impl BorderOverdraw {
    pub fn new(style: Style) -> Self {
        Self {
            style,
        }
    }
}

#[rustfmt::skip]
impl<'a> Widget for BorderOverdraw {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let txt = "┤ [F4] Help ├";
        let txt_w = txt.len();
        buf.set_span(area.x + area.width - txt_w as u16 - 1, area.y, &Span::styled(txt, self.style), txt_w as u16);
    }
}
