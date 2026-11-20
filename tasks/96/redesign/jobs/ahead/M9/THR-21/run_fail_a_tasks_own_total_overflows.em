# Clause THR-21.3: "Each task's accumulator has the place's type, and its operations are checked as
# the place's are, and so are the combining operations at the join (verdict 105): (...) a task whose
# own total overflows stops the program there (`[TYP-8]`), even where adding one by one would not
# (`[[-100], [100, 100]]` summed into an `i8` by two tasks stops in the second, at 100 + 100, where
# adding one by one gives 100)"
#$ test: run-fail
#$ rules: THR-21, TYP-8
#$ panics: integer overflow
# The spec's own case, with its own task line (Q4).

import std.thread

fn main():
    t: i8 = 0
    parts: Array[Array[i8]] = [[-100], [100, 100]]
    with scope = thread.scope():
        for part in parts:
            scope.spawn(owns fn() copy of part: for x in part: t += x)
    println(t)
