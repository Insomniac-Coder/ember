#$ test: run-pass
#$ rules: CORO-4, CORO-2
#$ profiles: debug, release
#$ stdout: (0, 0) (0, 1) (1, 0) (1, 1) (2, 0)
#$ ab-ab-
#$ 1 1 2 3 5 8
# `[CORO-4]` — the body becomes a state machine over the frame: the locals live across a
# `yield` (loop counters, a `String` being built, a running pair) are the frame's, kept from one
# step to the next through nested loops, `break`, `continue` and early `return`.

gen fn pairs(limit: int) -> Generator[(int, int)]:
    i = 0
    while true:
        for j in 0..2:
            if i == 2 and j == 1:
                return
            yield (i, j)
        i += 1

gen fn texts(times: int) -> Generator[String]:
    s: String = ""
    n = 0
    while n < times:
        n += 1
        if n % 3 == 0:
            continue
        s.push_str("ab-")
        if n == 2:
            yield s.clone()
            break

gen fn fibonacci() -> Generator[int]:
    a = 1
    b = 1
    while true:
        yield a
        a, b = b, a + b

fn main():
    out = [p for p in pairs(9)]
    println(out[0], out[1], out[2], out[3], out[4])
    for t in texts(5):
        println(t)
    fib = [x for x in fibonacci().take(6)]
    println(fib[0], fib[1], fib[2], fib[3], fib[4], fib[5])
