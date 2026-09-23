#$ test: run-pass
#$ rules: DRP-3
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ 2
#$ 1
# D-196 — a temporary made in one arm of a `match` or a conditional lives to
# the end of the statement and is dropped only on the path that made it.

fn width(s: str) -> int:
    return s.char_count()

fn main():
    r: Result[int, str] = Ok(1)
    v = match r:
        Ok(x) => x
        Err(e) => width(f"{e}")
    println(v)
    w = 1 if v == 0 else width(f"{v}!")
    println(w)
    failed: Result[int, str] = Err("x")
    u = match failed:
        Ok(x) => x
        Err(e) => width(f"{e}")
    println(u)
