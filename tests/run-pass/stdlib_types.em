#$ test: run-pass
#$ rules: ERR-1, ERR-2, LEX-19, STD-2

# `[ERR-1]` — `Option` and `Result` are ordinary payload enums, built by the
# compiler until Phase 2's generics let the standard library write them.
fn halve(n: i32) -> Option[i32]:
    if n % 2 == 0:
        return Some(n / 2)
    return None

# `[ERR-2]` — `?` yields the payload or returns the failure.
fn quarter(n: i32) -> Option[i32]:
    half = halve(n)?
    return halve(half)

fn checked(n: i32) -> Result[i32, i32]:
    if n < 0:
        return Err(n)
    return Ok(n * 2)

fn twice(n: i32) -> Result[i32, i32]:
    a = checked(n)?
    return Ok(a + a)

fn show(o: Option[i32]) -> i32:
    match o:
        Some(v):
            return v
        None:
            return -1

fn showr(r: Result[i32, i32]) -> i32:
    match r:
        Ok(v):
            return v
        Err(e):
            return -e

fn main():
    println(show(halve(10)))
    println(show(halve(7)))
    println(show(quarter(20)))
    println(show(quarter(6)))
    println(showr(twice(5)))
    println(showr(twice(-3)))

    # `Array[T]` — a growable sequence, bounds-checked like a fixed array.
    xs: Array[i32] = Array()
    for i in 0..5:
        xs.push(i * i)
    println(xs.len())
    println(xs[4])
    xs[2] = 99
    total = 0
    for i in 0..xs.len():
        total = total + xs[i]
    println(total)

    # `String` — the same buffer holding UTF-8 bytes.
    s = String()
    s.push_str("hello, ")
    s.push_str("world")
    println(s.len())
    println(s.as_str())

    # `[LEX-19]` — an f-string builds a `String`.
    name = "Ember"
    n = 42
    x = 1.5
    ok = true
    message = f"{name}: n={n} x={x} ok={ok}"
    println(message.as_str())
#$ stdout: 5
#$ -1
#$ 5
#$ -1
#$ 20
#$ 3
#$ 5
#$ 16
#$ 125
#$ 12
#$ hello, world
#$ Ember: n=42 x=1.5 ok=true
