//
//  Orb animation widget for tui-rs
//

use std::{time::{Instant, Duration}, sync::Arc};
use tui::{layout::Rect, buffer::Buffer, style::{Style, Color}, widgets::Widget, text::Span};

thread_local! {
    static LEVI: Arc<LeviRipple> = Arc::new(LeviRipple { start_time: Instant::now(), ms_count: 0 })
}

const TERM_COLOR_RAMP: &str = " .:-=+*#%@█";

pub struct LeviRipple {
    pub start_time: Instant,
    pub ms_count: u64,
}

fn char_from_intensity(intensity: u8) -> char {
    let w = (intensity as f32 / 255.).clamp(0., 1.);
    let m = (TERM_COLOR_RAMP.len() - 1) as f32 * w;
    TERM_COLOR_RAMP
        .chars()
        .nth(m.floor().clamp(2., 10.) as usize)
        .unwrap()
}

impl<'a> Widget for LeviRipple {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lev = LEVI.with(|z| z.clone());
        let time_elapsed = lev.start_time.elapsed();
        // Different Messages so it can always be centered
        let msg1 = "Waiting for Devil Daggers";
        let msg2 = "Waiting for Game";
        let mut tmp = [0; 4];
        let precalc = (time_elapsed.as_millis() / 200) as f32;
        let mut slp = 0;
        for y in 0..area.height {
            for x in 0..area.width {
                let map_x = -((area.width as f32 - x as f32) - (area.width as f32 / 2.));
                let map_y = (area.height as f32 - y as f32) - (area.height as f32 / 2.);
                let height = (((map_x * map_x + map_y * map_y).sqrt() - precalc) / 8.).sin();
                let height = (height * (255. / 2.)) + (255. / 2.);
                let height = height.clamp(20., 255.);
                buf.get_mut(x, y)
                    .set_symbol(char_from_intensity(height as u8).encode_utf8(&mut tmp))
                    .set_style(Style::default().bg(Color::Rgb(0, 0, 0)).fg(Color::Rgb(
                        height as u8,
                        0,
                        0,
                    )
                ));
            }
            slp += 1;
            if slp % 5 == 0 { std::thread::sleep(Duration::from_nanos(1)); }
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
