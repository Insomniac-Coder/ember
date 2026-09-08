#$ test: compile-fail
#$ rules: GRM-16, LEX-15, LEX-15a
#$ error[E0107]: a jump expression may not be an operand
#$ error[E0100]: expected an expression
#$ error[E0100]: expected an expression

## `[GRM-16]` — a jump is the whole of the expression it appears in.
fn f(a: i32) -> i32:
    return a + return a

## `[LEX-15]` / `[LEX-15a]` — `let` and `type` are keywords in every position,
## so neither is usable as a name without `r#`.
fn g():
    let: i32 = 1

fn h():
    type: i32 = 2
