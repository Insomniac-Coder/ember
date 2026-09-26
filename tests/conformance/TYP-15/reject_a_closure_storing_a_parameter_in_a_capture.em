#$ test: compile-fail
#$ rules: TYP-15, CLO-3, LT-42
# `[TYP-15]` (D-352) — a closure can be called where its caller cannot be
# checked, so it may store in a captured variable only a `static` view (or
# one of that variable's own); a view it was given is not one.

fn main():
    name = "ann"
    set = fn(v: str):
        name = v #$ error[E3063]: a parameter's view is stored in
    set("bob")
    println(name)
