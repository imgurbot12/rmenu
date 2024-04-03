#!/bin/sh

# customize rmenu options directly from stdin
rmenu-build options --placeholder 'placeholder'

# build entries for rmenu using rmenu-build tool to define json
rmenu-build entry -n foo -c 'run ls /' -a "`rmenu-build action ls / -c action-comment-1`"
rmenu-build entry -n foo -c 'run df -h' -a "`rmenu-build action -c baz -- df -h`"
rmenu-build entry -n baz -c 'run who' -a "`rmenu-build action who -c action-comment-3`"

# supports alternate action-modes like `echo` and `terminal`
rmenu-build entry -n hello -c 'echo helloworld' -a "`rmenu-build action --mode echo hello world!`"
rmenu-build entry -n term  -c 'runs top in term' -a "`rmenu-build action --mode terminal top`"
