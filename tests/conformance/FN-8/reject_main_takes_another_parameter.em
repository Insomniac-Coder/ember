#$ test: compile-fail
#$ rules: FN-8
#$ help: `main` takes nothing or `args: Array[String]`
# ODR-101 — `main`'s one parameter is the command line, an `Array[String]`;
# another was accepted, and the C compiler failed (D-551).

fn main(count: int): #$ error[E2183]: it takes `count: i64`
    println(count)
