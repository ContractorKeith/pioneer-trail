fn main() {
    for path in ["content.ron", "events", "quotes", "art"] {
        println!("cargo:rerun-if-changed={path}");
    }
}
