#!/bin/sh

# customize rmenu options directly from stdin
./target/release/rmenu-build.exe options --placeholder 'placeholder'

# build entries for rmenu using rmenu-build tool to define json
./target/release/rmenu-build.exe entry -n foo -c 'run ls /' -a "`./target/release/rmenu-build action ls / -c action-comment-1`"
./target/release/rmenu-build.exe entry -n foo -c 'run df -h' -a "`./target/release/rmenu-build action -c baz -- df -h`"
./target/release/rmenu-build.exe entry -n baz -c 'run who' -a "`./target/release/rmenu-build action who -c action-comment-3`"

# supports alternate action-modes like `echo` and `terminal`
./target/release/rmenu-build.exe entry -n hello -c 'echo helloworld' -a "`./target/release/rmenu-build action --mode echo hello world!`"
./target/release/rmenu-build.exe entry -n term  -c 'runs top in term' -a "`./target/release/rmenu-build action --mode terminal top`"
