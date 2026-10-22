#$ test: run-pass
#$ rules: DIA-15
#$ stdout: 3.0
# `[DIA-15]`: the import line offered for `sqrt` (the reject case beside this one) is the fix: with
# it applied, the program compiles and runs.
from std.math import sqrt

fn main():
    println(sqrt(9.0))
