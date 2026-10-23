#$ test: run-pass
#$ rules: CLO-3, CLO-6, CLO-6a
#$ profiles: debug, development, release
#$ stdout: drop a
#$ 1
#$ drop bb
#$ 2
#$ drop ccc
#$ 3
#$ end
# `[CLO-6]`, `[CLO-6a]` (D-880, D-910) — a callable value moved into a call ends exactly once: a
# `Box[fn]` and a `Box[once fn]` given to an owned parameter and called there, and a `once fn`
# given to one, each holding the only handle of a `Noisy`, which drops when its callable ends (the
# LLVM backend's port of D-910, tried against the C backend's).

class Noisy:
    name: String

    fn init(mut self, owned name: String):
        self.name = name

    fn drop(mut self):
        println("drop", self.name)

## A callable holding the only handle of a `Noisy`, which ends with it.
fn make(owned name: String) -> fn() -> int:
    n = Noisy(name)
    return owned fn() -> int => n.name.len()

fn make_once(owned name: String) -> once fn() -> int:
    n = Noisy(name)
    return owned fn() -> int => n.name.len()

fn call_box(owned b: Box[fn() -> int]) -> int:
    return b()

fn call_once_box(owned b: Box[once fn() -> int]) -> int:
    return b()

fn call_once(owned f: once fn() -> int) -> int:
    return f()

fn main():
    f: Box[fn() -> int] = Box(make("a".to_string()))
    println(call_box(f))
    g: Box[once fn() -> int] = Box(make_once("bb".to_string()))
    println(call_once_box(g))
    println(call_once(make_once("ccc".to_string())))
    println("end")
