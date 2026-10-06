#$ test: run-pass
#$ rules: CTL-2
#$ stdout: 12
#$ stdout: 12
#$ stdout: 18
#$ stdout: 9
#$ stdout: 12

## D-529 — an inner loop's iterator starts again on every turn of the loop
## around it. The view-hoisting pass moved the setting of an iterator's
## position out of the outer loop, because the step's write through
## `ref mut position` was not counted as a write: the inner loop then ran on
## the first turn only (`for c in s.chars()` inside `for _ in 0..4` pushed 25
## characters of 100).

fn chars_each_turn() -> int:
    n = 0
    for _ in 0..4:
        for _c in "abc".chars():
            n = n + 1
    return n

fn bytes_each_turn() -> int:
    n = 0
    for _ in 0..4:
        for _b in "abc".bytes():
            n = n + 1
    return n

fn list_each_turn() -> int:
    xs: Array[int] = [1, 2, 3]
    total = 0
    for _ in 0..3:
        for x in xs:
            total = total + x
    return total

fn enumerate_each_turn() -> int:
    n = 0
    for _ in 0..3:
        for (_i, _c) in "abc".chars().enumerate():
            n = n + 1
    return n

fn char_indices_each_turn() -> int:
    n = 0
    for _ in 0..4:
        for (_at, _c) in "abc".char_indices():
            n = n + 1
    return n

fn main():
    println(chars_each_turn())
    println(bytes_each_turn())
    println(list_each_turn())
    println(enumerate_each_turn())
    println(char_indices_each_turn())
