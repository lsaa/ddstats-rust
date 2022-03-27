//
//  Orb animation widget for tui-rs
//

use std::{time::Instant, sync::Arc};
use num_traits::Float;
use tui::{layout::Rect, buffer::Buffer, style::{Style, Color}, widgets::Widget, text::Span};

#[cfg(target_os = "windows")] use std::time::Duration;

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
    let m = TERM_COLOR_RAMP.chars().count() as f32 * w;
    TERM_COLOR_RAMP
        .chars()
        .nth(m.floor().clamp(0., TERM_COLOR_RAMP.chars().count() as f32 - 1.) as usize)
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
        let precalc = time_elapsed.as_millis() as f32 * (1. / 35.);
        #[cfg(target_os = "windows")] let mut slp = 0;
        for y in 0..area.height {
            for x in 0..area.width {
                let map_x = -((area.width as f32 - x as f32) - (area.width as f32 / 2.));
                let map_y = (area.height as f32 - y as f32) - (area.height as f32 / 2.);
                let height = (((map_x * map_x + map_y * map_y).sqrt() - precalc) / 30.).sin();
                let height = (height * (255. / 2.)) + (255. / 2.);
                let height = height.clamp(0., 255.);
                let a = 1.1;
                let b = height / (255.  * 3.);
                let a = a - (b * 0.9);
                let bitfuck = height.powf(a);
                let bitfuck = (bitfuck / 25.).floor() * 25.;
                let bitfuck = bitfuck.powf(1. / a);
                buf.get_mut(x, y)
                    .set_symbol(char_from_intensity(bitfuck as u8).encode_utf8(&mut tmp))
                    .set_style(Style::default().bg(Color::Rgb(0, 0, 0)).fg(Color::Rgb(
                        height as u8,
                        0,
                        0,
                    )
                ));
            }
            #[cfg(target_os = "linux")] std::thread::sleep(std::time::Duration::from_nanos(70000 * area.width as u64));
            #[cfg(target_os = "windows")] { slp += 1; }
            #[cfg(target_os = "windows")] if slp % 5 == 0 { std::thread::sleep(Duration::from_nanos(1)); }
        }

        let msg = if area.width % 2 == 0 {
            msg2
        } else {
            msg1
        };

        let mut s = "".to_owned();
        for _ in 0..msg.len() {
            s.push('#');
        }

        for _ in 0..16 {
            s.push('#');
        }

        buf.set_span(
            area.width / 2 - (msg.len() / 2) as u16 - 8,
            area.height / 2 - 1,
            &Span::styled(
                s.clone(),
                Style::default().bg(Color::Rgb(0,0,0)).fg(Color::Rgb(0,0,0)),
            ),
            msg.len() as u16 + 16,
        );
        buf.set_span(
            area.width / 2 - (msg.len() / 2) as u16 - 8,
            area.height / 2 + 1,
            &Span::styled(
                s.clone(),
                Style::default().bg(Color::Rgb(0,0,0)).fg(Color::Rgb(0,0,0)),
            ),
            msg.len() as u16 + 16,
        );
        buf.set_span(
            area.width / 2 - (msg.len() / 2) as u16 - 8,
            area.height / 2,
            &Span::styled(
                s.clone(),
                Style::default().bg(Color::Rgb(0,0,0)).fg(Color::Rgb(0,0,0)),
            ),
            msg.len() as u16 + 16,
        );
        buf.set_span(
            area.width / 2 - (msg.len() / 2) as u16,
            area.height / 2,
            &Span::styled(msg, Style::default().bg(Color::Rgb(0,0,0)).fg(Color::Rgb(255,255,255))),
            msg.len() as u16,
        );
    }
}
