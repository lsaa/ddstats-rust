//
//  Modular game data for UI
//

use ddcore_rs::models::{StatsBlockWithFrames, GameStatus, StatsFrame};
use num_traits::FromPrimitive;
use tui::{widgets::Row, style::Modifier, text::{Span, Spans}};
use crate::config;
use super::{ExtraSettings, SizeStyle};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum GameDataModules {
    RunData,
    Timer,
    Gems,
    Homing(SizeStyle), // Minimal, Compact, Full
    Kills,
    Accuracy,
    GemsLost(SizeStyle), // Minimal, Compact, Full
    CollectionAccuracy,
    HomingSplits(Vec<(String, f32, i32, i32, u32, Option<i32>)>), // (Name, Time, Offset, Positive threshold, Neutral zone, Golden Split)
    HomingUsed,
    DaggersEaten,
    FarmEfficiency,
    Spacing,
}

#[allow(unreachable_patterns, clippy::wildcard_in_or_patterns)] #[rustfmt::skip]
impl<'a> GameDataModules {
    pub fn to_rows(&'a self, data: &'a StatsBlockWithFrames, extra: &'a ExtraSettings) -> Vec<Row> {
        match self {
            GameDataModules::RunData => create_run_data_rows(data),
            GameDataModules::Timer => create_timer_rows(data),
            GameDataModules::Gems => create_gems_rows(data),
            GameDataModules::Homing(size_style) => create_homing_rows(data, size_style.clone()),
            GameDataModules::Kills => creake_kills_row(data),
            GameDataModules::Accuracy => create_accuracy_rows(data),
            GameDataModules::GemsLost(size_style) => create_gems_lost_rows(data, size_style.clone()),
            GameDataModules::CollectionAccuracy => create_collection_accuracy_rows(data),
            GameDataModules::HomingSplits(times) => create_homing_splits_rows(data, times.clone(), extra.clone()),
            GameDataModules::HomingUsed => create_homing_used_rows(data),
            GameDataModules::DaggersEaten => create_daggers_eaten_rows(data),
            GameDataModules::FarmEfficiency => create_farm_efficiency_rows(data),
            GameDataModules::Spacing | _ => vec![Row::new([""])],
        }
    }
}

fn create_run_data_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let styles = &config::cfg().ui_conf.theming.styles;
    let player = data.block.replay_player_username();
    let status = FromPrimitive::from_i32(data.block.status);
    let status_str = match status {
        Some(st) => match st {
            GameStatus::Menu => "MENU",
            GameStatus::Lobby => "LOBBY",
            GameStatus::Dead => crate::consts::DEATH_TYPES_CAPS[data.block.death_type as usize],
            GameStatus::Title => "TITLE",
            GameStatus::Playing => "ALIVE",
            GameStatus::LocalReplay => "LOCAL REPLAY",
            GameStatus::OtherReplay | GameStatus::OwnReplayFromLastRun | GameStatus::OwnReplayFromLeaderboard => "REPLAY",
        }.to_string(),
        None => "CONNECTING".to_string()
    };

    let status_span = if status == Some(GameStatus::Dead) {
        Span::styled(format!("   {}", status_str), styles.split_diff_neg.add_modifier(Modifier::BOLD))
    } else {
        Span::styled(format!("   {}", status_str), styles.text)
    };

    let player = Span::styled(player, styles.accent);
    vec![Row::new([status_span, player])]
}

fn create_timer_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let styles = &config::cfg().ui_conf.theming.styles;
    let timer_text = "   TIMER".to_string();
    let timer = if data.block.is_replay {
        format!("{:.4}/{:.4}", data.block.time, data.block.time_max + data.block.starting_time)
    } else {
        format!("{:.4}", data.block.time_max + data.block.starting_time)
    };
    let timer_text = Spans::from(vec![Span::styled(timer_text, styles.text)]);
    let timer = Spans::from(vec![Span::styled(timer, styles.accent), Span::styled("s", styles.text)]);
    vec![Row::new([timer_text, timer])]
}

fn create_gems_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let styles = &config::cfg().ui_conf.theming.styles;
    let gems_text = Spans::from(vec![Span::styled("   GEMS", styles.text)]);
    let gems = Spans::from(vec![Span::styled(format!("{}", data.block.gems_collected), styles.accent)]);
    vec![Row::new([gems_text, gems])]
}

fn get_homing(data: &StatsBlockWithFrames) -> u32 {
    if let Some(most_recent) = data.frames.last() {
        most_recent.homing as u32
    } else {
        0
    }
}

#[allow(unreachable_patterns)]
fn create_homing_rows(data: &StatsBlockWithFrames, style: SizeStyle) -> Vec<Row> {
    let styles = &config::cfg().ui_conf.theming.styles;
    let time = if data.block.time_lvl3 == 0. { 0. } else { data.block.time_max_homing };
    match style {
        SizeStyle::Full => {
            let homing_detail = Spans::from(vec![
                Span::styled(format!("{}", get_homing(data)), styles.accent),
                Span::styled(" [", styles.text),
                Span::styled(format!("{}", data.block.max_homing), styles.accent),
                Span::styled(" at ", styles.text),
                Span::styled(format!("{:.4}", time), styles.accent),
                Span::styled("s]", styles.text),
            ]);
            vec![Row::new([Spans::from(vec![Span::styled("   HOMING", styles.text)]), homing_detail])]
        }
        SizeStyle::Compact => {
            let homing_detail = Spans::from(vec![
                Span::styled(format!("{}", get_homing(data)), styles.accent),
                Span::styled(" [", styles.text),
                Span::styled(format!("{}", data.block.max_homing), styles.accent),
                Span::styled(" @ ", styles.text),
                Span::styled(format!("{:.4}", time), styles.accent),
                Span::styled("s]", styles.text),
            ]);
            vec![Row::new([Spans::from(vec![Span::styled("   HOMING", styles.text)]), homing_detail])]
        }
        SizeStyle::Minimal => {
            let homing_detail = Spans::from(vec![
                Span::styled(format!("{}", get_homing(data)), styles.accent)
            ]);
            vec![Row::new([Spans::from(vec![Span::styled("   HOMING", styles.text)]), homing_detail])]
        }
        _ => vec![],
    }
}

fn creake_kills_row(data: &StatsBlockWithFrames) -> Vec<Row> {
    let styles = &config::cfg().ui_conf.theming.styles;
    let kills_text = Spans::from(vec![Span::styled("   KILLS", styles.text)]);
    let kills = Spans::from(vec![Span::styled(format!("{}", data.block.kills), styles.accent)]);
    vec![Row::new([kills_text, kills])]
}

fn create_accuracy_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let styles = &config::cfg().ui_conf.theming.styles;
    if let Some(frame) = data.frames.last() {
        let mut acc = frame.daggers_hit as f32 / frame.daggers_fired as f32;
        let pacifist = frame.daggers_hit == 0;
        if acc.is_nan() {
            acc = 0.00;
        }

        let acc_text = Spans::from(vec![Span::styled("   ACCURACY", styles.text)]);
        let acc = if pacifist {
            Spans::from(vec![Span::styled("0.00".to_string(), styles.accent), 
                Span::styled("% [", styles.text), Span::styled("PACIFIST", styles.accent), Span::styled("]", styles.text)])
        } else {
            Spans::from(vec![Span::styled(format!("{:.2}", acc * 100.), styles.accent), Span::styled("%", styles.text)])
        };
        return vec![Row::new([acc_text, acc])];
    }
    
    let acc_text = Spans::from(vec![Span::styled("   ACCURACY", styles.text)]);
    let acc = Spans::from(vec![Span::styled("0.00".to_string(), styles.accent), 
        Span::styled("% [", styles.text), Span::styled("PACIFIST", styles.accent), Span::styled("]", styles.text)]);
    vec![Row::new([acc_text, acc])]
}

#[rustfmt::skip] #[allow(unreachable_patterns)]
fn create_gems_lost_rows(data: &StatsBlockWithFrames, style: SizeStyle) -> Vec<Row> {
    let styles = &config::cfg().ui_conf.theming.styles;
    let total_gems_lost = data.block.gems_eaten + data.block.gems_despawned;
    match style {
        SizeStyle::Full => {
            let gems_lost_detail = Spans::from(vec![
                Span::styled(format!("{}", total_gems_lost), styles.accent),
                Span::styled(" [", styles.text),
                Span::styled(format!("{}", data.block.gems_despawned), styles.accent),
                Span::styled(" DESPAWNED, ", styles.text),
                Span::styled(format!("{}", data.block.gems_eaten), styles.accent),
                Span::styled(" EATEN]", styles.text),
            ]);
            vec![Row::new([Spans::from(vec![Span::styled("   GEMS LOST", styles.text)]), gems_lost_detail])]
        },
        SizeStyle::Compact => {
            let gems_lost_compact = Spans::from(vec![
                Span::styled(format!("{}", total_gems_lost), styles.accent),
                Span::styled(" [", styles.text),
                Span::styled(format!("{}", data.block.gems_despawned), styles.accent),
                Span::styled(" + ", styles.text),
                Span::styled(format!("{}", data.block.gems_eaten), styles.accent),
                Span::styled("]", styles.text),
            ]);
            vec![Row::new([Spans::from(vec![Span::styled("   GEMS LOST", styles.text)]), gems_lost_compact])]
        },
        SizeStyle::Minimal => {
            let gems_lost_min = Spans::from(vec![Span::styled(format!("{}", total_gems_lost), styles.accent)]);
            vec![Row::new([Spans::from(vec![Span::styled("   GEMS LOST", styles.text)]), gems_lost_min])]
        },
        _ => vec![]
    }
}

fn create_collection_accuracy_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let styles = &config::cfg().ui_conf.theming.styles;
    let total_gems_lost = data.block.gems_eaten + data.block.gems_despawned;
    if let Some(frame) = data.frames.last() {
        let mut acc = (frame.gems_total - total_gems_lost) as f32 / frame.gems_total as f32;
        if acc.is_nan() {
            acc = 0.00;
        }

        let acc_text = Spans::from(vec![Span::styled("   COLLECTION ACC", styles.text)]);
        let acc = Spans::from(vec![Span::styled(format!("{:.2}", acc * 100.), styles.accent), Span::styled("%", styles.text)]);
        return vec![Row::new([acc_text, acc])];
    }
    
    let acc_text = Spans::from(vec![Span::styled("   COLLECTION ACC", styles.text)]);
    let acc = Spans::from(vec![Span::styled("0.00".to_string(), styles.accent), Span::styled("%", styles.text)]);
    vec![Row::new([acc_text, acc])]
}

fn create_homing_splits_rows(
    data: &StatsBlockWithFrames,
    times: Vec<(String, f32, i32, i32, u32, Option<i32>)>,
    extra: ExtraSettings,
) -> Vec<Row> {
    let real_timer = data.block.time_max + data.block.starting_time;
    let styles = &config::cfg().ui_conf.theming.styles;
    let cfg = crate::config::cfg();

    let current_split_idx = {
        if times.is_empty() { 0 } else {
            let mut res = 0;
            for (i, (_n, time, _o, _p, _neutral, _g)) in times.iter().enumerate() {
                if real_timer < *time {
                    res = i;
                    break;
                }
            }
            res
        }
    };

    times.iter().enumerate().filter_map(|(i, (name, time, offset, positive, neutral, gold))| {
        let split_text = "   SPLIT";

        if data.block.starting_time > *time || (real_timer <= *time && i != current_split_idx) || data.frames.is_empty() {
            if !extra.homing_always_visible {
                return None;
            }
            // "Empty" split
            return Some(Row::new([Spans::from(vec![Span::styled(split_text, styles.text)]), Spans::from(vec![
                Span::styled(format!("{:>4}: .... (    ) [   -   ]", name), styles.text),
            ])]));
        }

        if !(data.block.is_replay || data.block.is_in_game || extra.homing_always_visible) {
            return None;
        }

        let time_frame = data.get_frame_for_time(*time);
        let hom = if let Some(time_frame) = time_frame { time_frame.homing } else { data.frames.last().map_or(0, |x| x.homing) };
        let col = if let Some(time_frame) = time_frame { time_frame.gems_collected } else { data.frames.last().map_or(0, |x| x.gems_collected) };
        let arrow = crate::config::cfg().ui_conf.current_split_marker.clone();

        let v = StatsFrame::default();
        let last_split_frame = if i == 0 {
            Some(&v)
        } else {
            data.get_frame_for_time(times.get(i-1).unwrap().1)
        };

        let diff = {
            if let Some(frame) = last_split_frame {
                *offset - (frame.homing - hom)
            } else {
                *offset - (data.block.starting_homing - hom)
            }
        };

        let split_style = {
            let mut res = None;

            if let Some(g) = gold {
                if diff >= *g {
                    res = Some(styles.split_diff_gold);
                }
            }

            if res.is_none() && diff <= *positive + (*neutral as i32) && diff >= *positive - (*neutral as i32) {
                res = Some(styles.split_diff_neutral);
            } else if res.is_none() {
                if diff > *positive {
                    res = Some(styles.split_diff_pos);
                } else {
                    res = Some(styles.split_diff_neg)
                }
            }

            res.unwrap()
        };

        let collected = {
            if i == 0 {
                if let Some(time_frame) = data.get_frame_for_time(*time) {
                    time_frame.gems_collected
                } else {
                    data.block.gems_collected
                }
            } else if let Some(time_frame) = data.get_frame_for_time(times.get(i-1).unwrap().1) {
                col - time_frame.gems_collected
            } else {
                col
            }
        };

        let usage = {
            let used_homing_current = data.homing_usage_from_frames(Some(data.block.time_max));
            if i == 0 {
                 if let Some(_time_frame) = data.get_frame_for_time(*time) {
                    data.homing_usage_from_frames(Some(*time - data.block.starting_time)) as i32
                } else {
                    used_homing_current as i32
                }
            } else if let Some(_time_frame) = data.get_frame_for_time(*time) {
                let last_split_homing_usage = data.homing_usage_from_frames(Some(times.get(i-1).unwrap().1 - data.block.starting_time));
                data.homing_usage_from_frames(Some(*time - data.block.starting_time)) as i32 - last_split_homing_usage as i32
            } else {
                let last_split_homing_usage = data.homing_usage_from_frames(Some(times.get(i-1).unwrap().1 - data.block.starting_time));
                data.homing_usage_from_frames(Some(data.block.time_max)) as i32 - last_split_homing_usage as i32
            }
        };

        let is_live_split = i == current_split_idx && data.block.is_in_game && !data.block.is_replay;

        let split_name = {
            if cfg.ui_conf.current_split_live_change && is_live_split {
                format!("{:>4}", real_timer.floor() as i32)
            } else {
                format!("{:>4}", name)
            }
        };

        Some(Row::new([Spans::from(vec![Span::styled(split_text, styles.text)]), Spans::from(vec![
            Span::styled(format!("{:>4}", split_name), if is_live_split { styles.accent } else { styles.text }),
            Span::styled(": ".to_string(), styles.text),
            Span::styled(format!("{:>4}", hom), styles.accent),
            Span::styled(" (", styles.text),
            Span::styled(format!("{:<+4}", diff), split_style),
            Span::styled(") [", styles.text),
            Span::styled(format!("{:<3}", collected), styles.split_diff_pos),
            Span::styled("-", styles.text),
            Span::styled(format!("{:<3}", usage), styles.split_diff_neg),
            Span::styled("] ", styles.text),
            Span::styled(if is_live_split { arrow } else { "".to_string() }, styles.accent),
        ])]))
    }).collect()
}

fn create_homing_used_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let styles = &config::cfg().ui_conf.theming.styles;
    let homing_used_text = Spans::from(vec![Span::styled("   HOMING USED", styles.text)]);
    let homing_used = Spans::from(vec![Span::styled(format!("{}", data.homing_usage_from_frames(Some(data.block.time))), styles.accent)]);
    vec![Row::new([homing_used_text, homing_used])]
}

fn create_daggers_eaten_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let styles = &config::cfg().ui_conf.theming.styles;
    let daggers_eaten_text = Spans::from(vec![Span::styled("   DAGGERS EATEN", styles.text)]);
    let daggers_eaten = Spans::from(vec![Span::styled(format!("{}", data.block.daggers_eaten), styles.accent)]);
    vec![Row::new([daggers_eaten_text, daggers_eaten])]
}

fn create_farm_efficiency_rows(data: &StatsBlockWithFrames) -> Vec<Row> {
    let styles = &config::cfg().ui_conf.theming.styles;
    let farm_efficiency_text = Spans::from(vec![Span::styled("   FARM EFFICIENCY", styles.text)]);

    let mut farm_efficiency = 0.;

    if let Some(farm_end_frame) = data.frames.get(363) {
        if data.block.starting_time <= 0. {
            let used_by_farm_end = farm_end_frame.gems_collected - farm_end_frame.homing - 220;
            let kill_baseline = farm_end_frame.kills - (123 + 4) + (farm_end_frame.enemies_alive - 1);
            let elite_skull_collection = farm_end_frame.gems_collected - 285 - used_by_farm_end;
            let optimum_skull_count = elite_skull_collection as f32 * 11_f32 / kill_baseline as f32;
            farm_efficiency = optimum_skull_count * 100.;
        }
    }

    let farm_efficiency = Spans::from(vec![
        Span::styled(format!("{:.2}", farm_efficiency), styles.accent),
        Span::styled("%".to_string(), styles.text)
    ]);

    vec![Row::new([farm_efficiency_text, farm_efficiency])]
}
