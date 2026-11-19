#$ test: compile-fail
#$ rules: FN-3, LT-1
## `[FN-3]` — a returned reference must borrow a parameter the result may
## borrow from, or be `static`; a local's storage ends with the frame.

fn local() -> ref int:
    x = 5
    return x    #$ error[E3063]: this points into `x`

fn main():
    pass
