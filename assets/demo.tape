# Build first: cargo build --release -p pioneer-trail
# The recording uses a fresh save directory; it never replaces a player's journey.
Output assets/pioneer-trail.gif
Set Shell zsh
Set FontSize 16
Set Width 1000
Set Height 640
Set Padding 16
Set Framerate 20
Hide
Type 'pioneer_demo_dir=$(mktemp -d /tmp/pioneer-demo.XXXXXX); env -u NO_COLOR target/release/pioneer-trail --seed 41 --save-dir "$pioneer_demo_dir"'
Enter
Sleep 2s
Show
Sleep 2s
Enter
Sleep 500ms
Enter
Enter
Type "Ada"
Enter
Type "Ben"
Enter
Type "Clara"
Enter
Type "Daniel"
Enter
Type "Eve"
Enter
Enter
Enter
Sleep 1s
Type "?"
Sleep 3s
Type "?"
Right 2
Enter
Down
Right 12
Enter
Down
Right 4
Enter
Down
Right 9
Enter
Down
Enter
Down
Enter
Down
Enter
Down
Right
Enter
Down
Enter
Down 10
Enter
Sleep 2s
Type "7"
Sleep 4s
Left 8
Space
Sleep 1s
Right 16
Space
Sleep 2s
Escape
Type "a"
Sleep 4s
Ctrl+Q
