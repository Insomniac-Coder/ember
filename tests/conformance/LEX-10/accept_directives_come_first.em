#! module overflow(wrap)
#$ test: run-pass
#$ rules: LEX-10, GRM-37
#$ stdout: 1
## `[LEX-10]` - a directive above every import and item is a directive.

import std.io

fn main():
    println(1)
