#$ test: run-pass
#$ rules: LT-1
#$ stdout: 1
#$ a
# ODR-024 — a `mut` parameter of a `Copy` type is not a source: its value can
# be returned instead of viewed, so the result borrows only `src` and the
# caller may read `pos` while the token lives.

fn next_token(mut pos: int, src: str) -> str:
    pos += 1
    return src[0..1]

fn main():
    pos = 0
    token = next_token(pos, "abc")
    println(pos)
    println(token)
