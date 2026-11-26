# VHS Tape Script for treecut v0.4.0 (9:16 vertical reel)
# Run with: vhs reel.tape

Output assets/reel.gif

Set Shell "bash"
Set FontSize 18
Set Width 720
Set Height 1280
Set Padding 28
Set Theme "Catppuccin Mocha"
Set TypingSpeed 35ms
Set PlaybackSpeed 1.0

Hide
Type "mkdir -p assets && clear"
Enter
Sleep 300ms
Show

Type "# treecut v0.4.0 — AST context slicing for AI agents"
Enter
Sleep 700ms

Type "clear"
Enter
Sleep 200ms
Type "# 1) Compressed repo map + token savings"
Enter
Sleep 300ms
Type "treecut map --depth 2 --max-chars 900 --stats"
Enter
Sleep 3s

Type "clear"
Enter
Sleep 200ms
Type "# 2) Transitive call-graph slice (depth 2, both directions)"
Enter
Sleep 300ms
Type "treecut slice slice_symbol --depth 2 --direction both --max-chars 1400 --stats"
Enter
Sleep 4s

Type "clear"
Enter
Sleep 200ms
Type "# 3) Batch slice multiple symbols in one call"
Enter
Sleep 300ms
Type "treecut slice parse_file_with_imports extract_rust_imports --max-chars 1400 --stats"
Enter
Sleep 3.5s

Type "clear"
Enter
Sleep 200ms
Type "# 4) Git diff blast radius + callers"
Enter
Sleep 300ms
Type "treecut diff-slice --depth 2 --direction callers --max-chars 1400 --stats"
Enter
Sleep 4s
