#$ test: run-pass
#$ rules: TYP-47
#$ stdout: 1 1
#$ 1 1
#$ 3
## `[TYP-47]` - `Cs...` among type arguments (`View[Cs...]`), among a tuple type's elements (`(Entity, Cs...)`:
## with no type left it is `Entity` itself, with one a pair) and in a function type's parameters
## (`Array[fn(Ts...)]`) spells the pack's types out; `values...` passes the values on to the callable.

struct Entity:
    id: int

struct Position:
    x: int

[Cs...]
struct View:
    id: int

[Cs...]
fn view() -> View[Cs...]:
    return View[Cs...](1)

[Cs...]
fn tuple_of(x: (Entity, Cs...)) -> int:
    return 1

[Ts...]
fn listeners(ref mut f: Array[fn(Ts...)], values: Ts...):
    for g in f.iter_mut():
        g(values...)

fn show(a: int, b: int):
    println(a + b)

fn main():
    v = view[Position, Entity]()
    w = view[]()
    println(v.id, w.id)
    println(tuple_of[](Entity(1)), tuple_of[Position]((Entity(1), Position(2))))
    fs: Array[fn(int, int)] = [show]
    listeners(ref mut fs, 1, 2)
