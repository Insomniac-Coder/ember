#$ test: compile-fail
#$ rules: TYP-21
## `[TYP-21]` - `bool` and `char` implement none of the operator interfaces.

fn main():
    a = true
    b = a + a                        #$ error[E2020]: `+` cannot be applied to `bool`
    c = 'x'
    d = c + c                        #$ error[E2020]: `+` cannot be applied to `char`
    println(b, d)
