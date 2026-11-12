#$ test: compile-fail
#$ rules: STD-6, BLD-11
## `[STD-6]` (D-707) - the command line belongs to the process, which the `io` layer holds with
## time and files: `args` needs `io` as well as `alloc`. This directory's manifest declares `["core"]`.

import std.process

pub fn first() -> int:
    items = process.args()    #$ error[E1021]: `first` needs the `io` layer, which is not linked in this build
#$ error[E1021]: `first` needs the `alloc` layer, which is not linked in this build
    return items.len()
