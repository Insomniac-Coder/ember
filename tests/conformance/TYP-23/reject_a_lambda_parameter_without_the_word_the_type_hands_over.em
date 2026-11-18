#$ test: compile-fail
#$ rules: TYP-23
## `[TYP-23]` - a lambda parameter's word is what is written on it, never inferred, and matches the callable
## type it is given to: where the type hands the value over (`fn(owns String)`) the lambda writes `owns`.

fn each_name(names: Array[String], f: fn(owns String)):
    pass

fn main():
    each_name(["a"], fn(s) => println(s.len()))    #$ error[E2020]: the callable type hands each value over, so this function receives it as its own: write `owns s`
    each_name(["a"], fn(owns s) => println(s.len()))
