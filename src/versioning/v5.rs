//
// Handle updating all versioned structs to v5
//

impl From<DDStatsRustConfig!["1.0.0"]> for DDStatsRustConfig!["5.0.0"] {
    fn from(cfg: DDStatsRustConfig!["1.0.0"]) -> Self {
        let stream: <Stream as obake::Versioned>::Versioned = cfg.stream.into();
        let submit: <Submit as obake::Versioned>::Versioned = cfg.submit.into();
        let discord: <Discord as obake::Versioned>::Versioned = cfg.discord.into();
        let ui_conf: <UiConf as obake::Versioned>::Versioned = cfg.ui_conf.into();

        Self { 
            saved_games_max: 25,
            record_threshold: 350.,

            host: cfg.host,
            grpc_host: cfg.grpc_host,
            offline: cfg.offline,
            debug_logs: cfg.debug_logs,
            auto_clipboard: cfg.auto_clipboard,
            linux_restart_as_child: cfg.linux_restart_as_child,
            use_linux_proton: cfg.use_linux_proton,
            tray_icon: cfg.tray_icon,
            hide_window_on_start: cfg.hide_window_on_start,
            upload_replays_automatically: cfg.upload_replays_automatically,
            block_marker_override: cfg.block_marker_override,
            process_name_override: cfg.process_name_override,
            open_game_on_replay_request: cfg.open_game_on_replay_request,
            stream: <Stream!["5.0.0"]>::from(stream),
            submit: <Submit!["5.0.0"]>::from(submit),
            discord: <Discord!["5.0.0"]>::from(discord),
            ui_conf: <UiConf!["5.0.0"]>::from(ui_conf),
        }
    }
}

impl From<Stream!["1.0.0"]> for Stream!["5.0.0"] {
    fn from(cfg: Stream!["1.0.0"]) -> Self {
        Self {
            stats: cfg.stats,
            replay_stats: cfg.replay_stats,
            non_default_spawnsets: cfg.non_default_spawnsets,
        }
    }
}

impl From<Submit!["1.0.0"]> for Submit!["5.0.0"] {
    fn from(cfg: Submit!["1.0.0"]) -> Self {
        Self {
            stats: cfg.stats,
            replay_stats: cfg.replay_stats,
            non_default_spawnsets: cfg.non_default_spawnsets,
            ddcl: cfg.ddcl,
        }
    }
}

impl From<Discord!["1.0.0"]> for Discord!["5.0.0"] {
    fn from(cfg: Discord!["1.0.0"]) -> Self {
        Self {
            notify_player_best: cfg.notify_player_best,
            notify_above_1000: cfg.notify_above_1000,
            notify_custom_spawnsets: cfg.notify_custom_spawnsets,
        }
    }
}

impl From<UiConf!["1.0.0"]> for UiConf!["5.0.0"] {
    fn from(cfg: UiConf!["1.0.0"]) -> Self {
        let theming: <Theming as obake::Versioned>::Versioned = cfg.theming.into();

        Self {
            enabled: cfg.enabled,
            logo_style: cfg.logo_style,
            hide_logs: cfg.hide_logs,
            logo: cfg.logo,
            orb_connection_animation: cfg.orb_connection_animation,
            column_distance: cfg.column_distance,
            show_help_on_border: cfg.show_help_on_border,
            current_split_marker: cfg.current_split_marker,
            current_split_live_change: cfg.current_split_live_change,
            always_show_splits: cfg.always_show_splits,
            game_data_modules: cfg.game_data_modules,
            theming: <Theming!["5.0.0"]>::from(theming),
        }
    }
}

impl From<Theming!["1.0.0"]> for Theming!["5.0.0"] {
    fn from(cfg: Theming!["1.0.0"]) -> Self {
        let styles: <Styles as obake::Versioned>::Versioned = cfg.styles.into();

        Self {
            styles: <Styles!["5.0.0"]>::from(styles),
        }
    }
}

impl From<Styles!["1.0.0"]> for Styles!["5.0.0"] {
    fn from(cfg: Styles!["1.0.0"]) -> Self {
        Self {
            text: cfg.text,
            logo: cfg.logo,
            logs: cfg.logs,
            logs_title: cfg.logs_title,
            log_text: cfg.log_text,
            most_recent_log: cfg.most_recent_log,
            game_data: cfg.game_data,
            game_data_title: cfg.game_data_title,
            split_name: cfg.split_name,
            accent: cfg.accent,
            split_diff_pos: cfg.split_diff_pos,
            split_diff_neg: cfg.split_diff_neg,
            split_diff_neutral: cfg.split_diff_neutral,
            split_diff_gold: cfg.split_diff_gold,
        }
    }
}
