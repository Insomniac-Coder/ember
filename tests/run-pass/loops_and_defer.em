#$ test: run-pass
#$ rules: CTL-3, CTL-4, CTL-6, CTL-7, CTL-8
#$ assert-c: !contains("struct em_Range")

# `[CTL-3]` — a range loop is a counted loop, with no iterator object.
fn sum(n: i32) -> i32:
    total = 0
    for i in 0..n:
        total = total + i
    return total

fn inclusive(n: i32) -> i32:
    total = 0
    for i in 1..=n:
        total = total + i
    return total

# `continue` has to run the increment, or the loop never advances.
fn skipping() -> i32:
    seen = 0
    for i in 0..10:
        if i == 3:
            continue
        seen = seen + 1
    return seen

# `[CTL-4]` — `else` runs when the loop ends on its own, not on `break`.
fn first_three(limit: i32) -> i32:
    for i in 0..limit:
        if i == 3:
            return i
    else:
        return -1
    return -2

fn broken() -> i32:
    while true:
        break
    else:
        return 1
    return 0

# A label names which loop to leave.
fn labelled() -> i32:
    hits = 0
    outer: for i in 0..3:
        for j in 0..3:
            if j == 1:
                continue outer
            if i == 2:
                break outer
            hits = hits + 1
    return hits

# `[CTL-7]`, `[CTL-8]` — deferred blocks run at scope exit, last first, and a
# `return` runs them before it leaves.
fn ordered():
    println(1)
    defer:
        println(4)
    defer:
        println(3)
    println(2)

fn deferred_return() -> i32:
    defer:
        println(9)
    return 7

fn deferred_in_loop():
    for i in 0..2:
        defer:
            println(i)
        println(100)

# `[CTL-6]` — `with` binds for the block.
fn scoped() -> i32:
    with x = 41, y = 1:
        return x + y
    return 0

fn main():
    println(sum(5))
    println(inclusive(4))
    println(skipping())
    println(first_three(10))
    println(first_three(2))
    println(broken())
    println(labelled())
    ordered()
    println(deferred_return())
    deferred_in_loop()
    println(scoped())
#$ stdout: 10
#$ 10
#$ 9
#$ 3
#$ -1
#$ 0
#$ 2
#$ 1
#$ 2
#$ 3
#$ 4
#$ 9
#$ 7
#$ 100
#$ 0
#$ 100
#$ 1
#$ 42
