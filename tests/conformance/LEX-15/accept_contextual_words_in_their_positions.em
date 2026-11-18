#$ test: run-pass
#$ rules: LEX-15
#$ stdout: 3
#$ stdout: 4
#$ stdout: 5
#$ stdout: 3
#$ stdout: 2
#$ stdout: 3
## `[LEX-15]` — the contextual keywords in the position the rule states: `abstract` and `open` before
## `class`, `gen` immediately before `fn`, `once` before `fn` in a type, `some` at the start of a type,
## `package` and `read` inside the brackets after `pub`, `copy` directly before `of`, `exists` between
## the value and `as` in an `if` header.

abstract class Shape:
    fn area() -> int:
        return 0

open class Base:
    n: int = 1

gen fn count(n: int) -> Generator[int]:
    for i in 0..n:
        yield i

fn run(f: once fn() -> int) -> int:
    return f()

fn pick() -> some Display:
    return 5

struct Cfg:
    pub(package, read) size: int = 3

fn main():
    total = 0
    for v in count(3):
        total += v
    println(total)
    println(run(owns fn(): 4))
    println(pick())
    c = Cfg()
    println(c.size)
    o: Option[int] = Some(2)
    if o exists as k:
        println(k)
    z = copy of total
    println(z)
