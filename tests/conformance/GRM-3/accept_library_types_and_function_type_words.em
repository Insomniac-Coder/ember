#$ test: run-pass
#$ rules: GRM-3
#$ stdout: 4
## `[GRM-3]` — `Array[T]`, `Map[K, V]`, `Option[T]`, `Result[T, E]`, `Span[T]` are ordinary `path_type`s; a
## function type's parameter words are the parameters' own (`fn(ref mut Ui)`, `fn(owns String) -> int`).

class Ui:
    n: int = 0

fn bump(ref mut u: Ui):
    u.n += 1

fn eat(owns s: String) -> int:
    return s.len()

fn run(f: fn(ref mut Ui), g: fn(owns String) -> int) -> int:
    u = Ui()
    f(ref mut u)
    return g(owns String("abc")) + u.n

fn main():
    m: Map[String, Array[int]] = Map()
    t: Result[int, String] = Ok(1)
    println(run(bump, eat) + m.len() + t.unwrap() - 1)
