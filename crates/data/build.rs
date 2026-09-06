fn main() {
    // Directory watching also catches newly added RON/PX files, not just edits.
    for path in ["content.ron", "events", "quotes", "art"] {
        println!("cargo:rerun-if-changed={path}");
    }
}
