use crate::Stats;
use std::fs::read_to_string;

pub fn load_stats() -> Stats {
    let stats_json = read_to_string("stats.json").unwrap();

    let stats: Stats = serde_json::from_str(stats_json.as_str()).unwrap();

    stats
}