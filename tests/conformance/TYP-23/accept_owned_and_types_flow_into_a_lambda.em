#$ test: run-pass
#$ rules: TYP-23, CLO-3
#$ stdout: Some(3)
#$ xy!
#$ 3
#$ Some(6)
# ODR-025 — expected types flow into a lambda (`[TYP-23]`), and so does an
# `owned` parameter mode: `fn(s) => s` against `fn(owned T) -> U` receives `s`
# owned. Generic inference reads `Option[T]` against `Option[String]`, and a
# ternary branch `None` takes its type from the other branch.

[T, U]
fn opt_map(owns o: Option[T], f: fn(owns T) -> U) -> Option[U]:
    match owns o:
        Some(v):
            return Some(f(v))
        None:
            return None

[T, U]
fn apply(owns x: T, f: fn(owns T) -> U) -> U:
    return f(x)

[T]
fn unwrap2(owns o: Option[T]) -> T:
    match owns o:
        Some(v):
            return v
        None:
            panic("none")

fn main():
    s: Option[String] = Some("abc")
    println(opt_map(owns s, fn(v) => v.len()))
    suffix: String = "!"
    t: String = "xy"
    println(apply(owns t, fn(v) => v + suffix))
    w: Option[String] = Some("abc")
    println(unwrap2(owns w).len())
    v = 5
    println(Some(v + 1) if v > 3 else None)
