#$ test: run-pass
#$ rules: TYP-1
#$ stdout: ok
## `[TYP-1]` - a concrete type's `Copy` and `Send` properties decide which bounds it meets.

@derive(Copy)
struct Pair:
    a: int
    b: int

[T: needs Copy]
fn copy_it(x: T) -> T:
    return x

[T: needs Send]
fn send_it(owns x: T) -> T:
    return x

fn main():
    p = copy_it(Pair(1, 2))
    q = send_it(owns String("hi"))
    println("ok" if p.a == 1 and q.len() == 2 else "bad")
