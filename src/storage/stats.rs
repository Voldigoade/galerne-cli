//! Statistics formatting and tracking for the player's career.

use crate::storage::schema::SaveState;

/// Formats career statistics into a human-readable text block.
pub fn format_career_stats(state: &SaveState) -> String {
    let play_h = state.total_play_time_secs / 3600;
    let play_m = (state.total_play_time_secs % 3600) / 60;
    let play_s = state.total_play_time_secs % 60;

    let wait_h = state.total_wait_time_saved_secs / 3600;
    let wait_m = (state.total_wait_time_saved_secs % 3600) / 60;
    let wait_s = state.total_wait_time_saved_secs % 60;

    format!(
        "Total Wait Time Saved:  {:02}h {:02}m {:02}s\n\
         Total Game Time:        {:02}h {:02}m {:02}s\n\
         Runs Initiated:         {}\n\
         Distance Traveled:      {:.0}m\n\
         Gale Shards in Bank:    {}",
        wait_h,
        wait_m,
        wait_s,
        play_h,
        play_m,
        play_s,
        state.total_runs_started,
        state.total_distance_traveled,
        state.currency_shards
    )
}
