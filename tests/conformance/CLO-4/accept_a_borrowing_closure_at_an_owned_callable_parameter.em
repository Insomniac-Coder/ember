#$ test: compile-fail
#$ rules: CLO-1, CLO-2, CLO-3, CLO-4, TYP-15
# An `owned f` parameter may retain the callable after its caller returns.
# Moving a normal closure into it would make the closure's captured reference
# outlive the local it borrowed, even when this particular body only invokes it.

fn invoke(owned f: fn() -> i32) -> i32:
    return f()

fn main():
    value: i32 = 7
    closure = fn() => value
    println(invoke(closure)) #$ error[E3063]: captures by reference cannot be passed to an `owned` callable parameter
