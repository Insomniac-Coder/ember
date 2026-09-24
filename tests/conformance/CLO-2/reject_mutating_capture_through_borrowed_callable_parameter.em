#$ test: compile-fail
#$ rules: CLO-2, CLO-3, FN-1, BRW-1, TST-19

# A mutable-capturing closure cannot be invoked through the borrowed `f`
# parameter. The declaration must use the ordinary `mut f` mode instead.

fn invoke(f: fn() -> i32) -> i32:
    return f() #$ error[E3023]: cannot mutate borrowed parameter `f`

fn main():
    counter: i32 = 0
    increment = fn() -> i32:
        counter = counter + 1
        return counter
    println(invoke(increment))
