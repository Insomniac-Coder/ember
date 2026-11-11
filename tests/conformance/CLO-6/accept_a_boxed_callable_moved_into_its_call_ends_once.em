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

    fn init(owns name: String):
        this.name = name

    fn drop():
        println("drop", this.name)

## A callable holding the only handle of a `Noisy`, which ends with it.
fn make(owns name: String) -> fn() -> int:
    n = Noisy(name)
    return owned fn() -> int => n.name.len()

fn make_once(owns name: String) -> once fn() -> int:
    n = Noisy(name)
    return owned fn() -> int => n.name.len()

fn call_box(owns b: owns fn() -> int) -> int:
    return b()

fn call_once_box(owns b: owns once fn() -> int) -> int:
    return b()

fn call_once(owns f: once fn() -> int) -> int:
    return f()

fn main():
    f: owns fn() -> int = owns make(owns "a".to_string())
    println(call_box(owns f))
    g: owns once fn() -> int = owns make_once(owns "bb".to_string())
    println(call_once_box(owns g))
    println(call_once(owns make_once(owns "ccc".to_string())))
    println("end")
