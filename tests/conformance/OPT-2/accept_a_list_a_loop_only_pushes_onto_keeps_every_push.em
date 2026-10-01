#$ test: run-pass
#$ rules: OPT-2, PHIL-5
#$ profiles: debug, release, shipping
#$ stdout:
#$ 5 [0, 1, 2, 3, 4] stopped
#$ 7 [10, 11, 12, 13, 14, 15, 16] done
#$ [0, 2, 4, 6, 8] [1, 3, 5, 7, 9]
#$ 1 2 3 4
#$ 5 6 7 8
#$ 6 6000
#$ [(1, 2), (2, 4), (3, 6)]
# A list a loop only pushes onto is kept in a local for the loop and written
# back on every way out of it (`list_locals.rs`); the results are the same as
# before. A list in an object, left by `break` and by the loop's end (each
# writing back, the `else` only on the end), two lists one loop pushes onto,
# a loop where another handle to the same object reads the list as it grows
# and one where a function does (neither kept in a local: each sees every
# push), a loop copied by bounds-check versioning (`[OPT-2]`) pushing from a
# list it indexes, and pushes of non-scalar values.

class Log:
    items: Array[int]

    fn init(mut self):
        self.items = []

fn count(log: Log) -> int:
    return len(log.items)

fn fill(mut log: Log, start: int, stop: int, limit: int):
    for i in start..stop:
        if i == start + limit:
            break
        log.items.push(i)
    else:
        print(len(log.items), log.items, "done")
        println()
        return
    print(len(log.items), log.items, "stopped")
    println()

fn main():
    a = Log()
    fill(a, 0, 10, 5)
    b = Log()
    fill(b, 10, 17, 100)

    evens: Array[int] = []
    odds: Array[int] = []
    for i in 0..10:
        if i % 2 == 0:
            evens.push(i)
        else:
            odds.push(i)
    println(evens, odds)

    c = Log()
    same = c
    for i in 0..4:
        c.items.push(i)
        print(len(same.items))
        if i < 3:
            print(" ")
    println()
    for i in 0..4:
        c.items.push(i)
        print(count(c))
        if i < 3:
            print(" ")
    println()

    source: Array[int] = [1000, 1000, 1000, 1000, 1000, 1000]
    d = Log()
    total = 0
    for i in 0..len(source):
        d.items.push(source[i])
        total += source[i]
    println(len(d.items), total)

    pairs: Array[(int, int)] = []
    for i in 1..4:
        pairs.push((i, i * 2))
    println(pairs)
