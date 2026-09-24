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

fn opt_map[T, U](owned o: Option[T], f: fn(owned T) -> U) -> Option[U]:
    match o:
        Some(v):
            return Some(f(v))
        None:
            return None

fn apply[T, U](owned x: T, f: fn(owned T) -> U) -> U:
    return f(x)

fn unwrap2[T](owned o: Option[T]) -> T:
    match o:
        Some(v):
            return v
        None:
            panic("none")

fn main():
    s: Option[String] = Some("abc")
    println(opt_map(s, fn(v) => v.len()))
    suffix: String = "!"
    t: String = "xy"
    println(apply(t, fn(v) => v + suffix))
    w: Option[String] = Some("abc")
    println(unwrap2(w).len())
    v = 5
    println(Some(v + 1) if v > 3 else None)
