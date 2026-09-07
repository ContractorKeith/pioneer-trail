//! Text presentation for the optional sealed-letter errand.

pub fn reward(cents: i64) -> String {
    format!("${:.2}", cents as f64 / 100.0)
}
