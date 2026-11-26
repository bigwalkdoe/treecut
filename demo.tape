# VHS Tape Script for treecut demo
# Run with: vhs demo.tape

Output assets/demo.gif

# Terminal Window & Theme Styling
Set Shell "bash"
Set FontSize 15
Set Width 1150
Set Height 680
Set Padding 24
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set PlaybackSpeed 1.0

# Ensure assets directory exists and warm up silently
Hide
Type "mkdir -p assets"
Enter
Sleep 200ms
Type "clear"
Enter
Show

# 1. Compressed Repository Skeleton
Type "# 1. Generate a compressed, token-budgeted repo skeleton"
Enter
Sleep 400ms
Type "treecut map --depth 2 --max-chars 1400"
Enter
Sleep 2.5s

# 2. Surgical Symbol + Caller Slicing
Type "clear"
Enter
Sleep 200ms
Type "# 2. Slice a symbol's definition + every cross-file call site"
Enter
Sleep 400ms
Type "treecut slice slice_symbol --max-chars 1800"
Enter
Sleep 3s

# 3. Git-Diff Blast Radius
Type "clear"
Enter
Sleep 200ms
Type "# 3. Extract modified AST symbols in Git diff + impacted callers"
Enter
Sleep 400ms
Type "treecut diff-slice --max-chars 1800"
Enter
Sleep 3.5s