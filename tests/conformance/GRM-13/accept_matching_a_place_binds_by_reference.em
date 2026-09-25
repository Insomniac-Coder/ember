#$ test: run-pass
#$ rules: GRM-13, GRM-15
#$ stdout: 3
#$ stdout: Some('abc')
#$ stdout: 2 x
#$ stdout: Some('xy')
#$ stdout: abc!
# `[GRM-13]` — matching a place that is not consumed binds `Copy` fields by
# value and other fields by reference, so the place is still whole afterwards:
# a local, a borrowed parameter, an array element and a field. `ref mut`
# borrows mutably. `match owned e:` consumes `e` and binds by move, and a
# scrutinee that is not a place is a temporary, so its parts move too.

struct Tagged:
    n: int
    name: String

fn length(o: Option[String]) -> int:
    match o:
        Some(s): return s.len()
        None: return 0

fn take(owned s: String) -> String:
    return s + "!"

fn make() -> Option[String]:
    return Some("abc")

fn main():
    a: Option[String] = Some("abc")
    match a:
        Some(s): println(s.len())
        None: pass
    println(a)
    items = [Tagged(2, "x")]
    match items[0]:
        Tagged(n = n, name = name): println(n, name)
    b: Option[String] = Some("x")
    match b:
        Some(ref mut s): s += "y"
        None: pass
    println(b)
    match owned a:
        Some(s): println(take(s))
        None: pass
    match make():
        Some(s): take(s)
        None: pass
    _ = length(b)
