#$ test: compile-fail
#$ rules: CLO-3, CLO-6, OWN-3, TST-19

# A callable reached through `owned f: fn(...)` is `CallableOnce`: its first
# call consumes the parameter binding, so a second call is use after move.

fn twice(owned f: fn() -> i32) -> i32:
    _first = f()
    return f() #$ error[E3040]: `f` has been moved out of

fn main():
    println(twice(fn() => 7))
