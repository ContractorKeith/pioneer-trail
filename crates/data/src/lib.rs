//! Content loader. All game content lives beside this crate as RON and `.px` art,
//! embedded into the binary at build time.

use include_dir::{include_dir, Dir};

pub static TRAILS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/trails");
pub static EVENTS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/events");
pub static ART: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/art");

/// Names of every embedded trail file. Placeholder until Phase 1 defines the schema.
pub fn trail_files() -> Vec<&'static str> {
    TRAILS.files().filter_map(|f| f.path().to_str()).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn oregon_trail_is_embedded() {
        assert!(super::trail_files().iter().any(|f| f.ends_with("oregon.ron")));
    }
}
