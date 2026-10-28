#$ test: compile-fail
#$ rules: LEX-14
## design rules: R15
## R15: "`async` stays reserved (E0005)" (AN-77's message).

async fn fetch() -> int:    #$ error[E0005]: `async` is planned for a later version; for work that waits, use a generator or threads (`with scope = thread.scope():`)
    return 1

fn main():
    println(1)
