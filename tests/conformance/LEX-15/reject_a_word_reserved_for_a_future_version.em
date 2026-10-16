#$ test: compile-fail
#$ rules: LEX-15, LEX-14
# `[LEX-15]` — `async`, `await` and `macro` are reserved for a future version: one used as a name
# is `E0005`, naming the version (`[LEX-14]`).

fn main():
    async = 1    #$ error[E0005]: `async` is reserved for v2
    println(async)    #$ error[E0005]: `async` is reserved for v2
