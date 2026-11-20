# Clause CLO-3.1: "`fn(A) -> R` as the type of a parameter, a field, a local, a result, a
# collection's element or a `static` is a function value of that type (`[CLO-10]`), filled from a
# named function (...), a local function or a lambda"; "A parameter of callable type receives a
# function value and calls through it"; "A long callable type is named once with `type` (`type
# Handler = fn()`, then `handlers: Array[Handler] = []` and `fn on_click(owns f: Handler):`)"
#$ test: run-pass
#$ rules: CLO-3, CLO-10, CLO-1
#$ stdout: hello
#$ stdout: lambda
#$ stdout: 6
#$ stdout: 9
# The spec's `Handler`, a parameter, a result and a `static` of callable type.

type Handler = fn()

static mut HANDLERS: Array[Handler] = []
static TIMES3: fn(int) -> int = times3

fn hello():
    println("hello")

fn times3(x: int) -> int:
    return x * 3

fn on_click(owns f: Handler):
    HANDLERS.push(owns f)

fn run_with(f: fn(int) -> int, x: int) -> int:
    return f(x)

fn chosen() -> fn(int) -> int:
    return times3

fn main():
    on_click(owns hello)
    on_click(owns fn(): println("lambda"))
    for ref mut h in HANDLERS:
        h()
    println(run_with(fn(x: int) -> int => x * 3, 2))
    println(chosen()(3) + TIMES3(0))
