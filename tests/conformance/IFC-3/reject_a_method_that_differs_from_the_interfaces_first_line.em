#$ test: compile-fail
#$ rules: IFC-3, DIA-2, DIA-25
# `[IFC-3]` — an implementation writes each method with the interface's first line after the name:
# the same parameter names, words and types and the same result. One that differs is `E2040`,
# naming the method, both sides and the line to write.

interface Scaler:
    fn scale(k: f64) -> f64

interface Greeter:
    fn greet(who: StrView) -> int

interface Mover:
    fn go(ref mut x: int)

interface Getter:
    fn get() -> int

struct Sq:
    side: f64

struct P:
    n: int

struct M:
    n: int

struct G:
    n: int

extend Sq implements Scaler:    #$ error[E2040]: `Sq.scale` takes `k: i64`, and `Scaler.scale` takes `k: f64`; write `fn scale(k: f64) -> f64:`
    fn scale(k: i64) -> f64:
        return 1.0

extend P implements Greeter:    #$ error[E2040]: `P.greet` names its parameter `person`, and `Greeter.greet` names it `who`; write `fn greet(who: StrView) -> int:`
    fn greet(person: StrView) -> int:
        return 1

extend M implements Mover:    #$ error[E2040]: `M.go` takes `x: int`, and `Mover.go` takes `ref mut x: int`; write `fn go(ref mut x: int):`
    fn go(x: int):
        pass

extend G implements Getter:    #$ error[E2040]: `G.get` returns `i32`, and `Getter.get` returns `i64`; write `fn get() -> int:`
    fn get() -> i32:
        return 1

fn main():
    pass
