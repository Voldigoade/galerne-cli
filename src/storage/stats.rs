//! Statistics formatting and tracking for the player's career.

use crate::storage::schema::SaveState;
use crate::ui::i18n::{t, Locale};

/// Formats career statistics into a human-readable text block.
pub fn format_career_stats(state: &SaveState, locale: Locale) -> String {
    let play_h = state.total_play_time_secs / 3600;
    let play_m = (state.total_play_time_secs % 3600) / 60;
    let play_s = state.total_play_time_secs % 60;

    let wait_h = state.total_wait_time_saved_secs / 3600;
    let wait_m = (state.total_wait_time_saved_secs % 3600) / 60;
    let wait_s = state.total_wait_time_saved_secs % 60;

    format!(
        "{}: {:02}h {:02}m {:02}s\n\
         {}: {:02}h {:02}m {:02}s\n\
         {}: {}\n\
         {}: {:.0}m\n\
         {}: {}",
        t("wait_saved", locale),
        wait_h,
        wait_m,
        wait_s,
        t("play_time", locale),
        play_h,
        play_m,
        play_s,
        t("runs_started", locale),
        state.total_runs_started,
        t("distance_traveled", locale),
        state.total_distance_traveled,
        t("shards_collected", locale),
        state.currency_shards
    )
}
