#$ test: compile-fail
#$ rules: GRM-2
#$ error[E0100]: imports come first: move `import std.math` above line 6
# An import below an item is `E0100`; the message names the line to move it above.

fn main():
    println(1)

import std.math
