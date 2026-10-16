#$ test: compile-fail
#$ rules: LEX-15
# ODR-106 — `not` and `is` are method names only: a function standing alone, an item or a local
# function, may not take either (`E0005`); `r#is` is its raw spelling.

fn is() -> int:    #$ error[E0005]: `is` is a reserved word; only a method may be named `is`
    return 1

fn not() -> int:    #$ error[E0005]: `not` is a reserved word; only a method may be named `not`
    return 2

fn main():
    fn is():    #$ error[E0005]: `is` is a reserved word; only a method may be named `is`
        pass
