#$ test: compile-fail
#$ rules: LT-1a, LT-4b

#$ error[E2031]: `from` names `count`, which a result cannot point into
fn first(count: usize, values: Span[i32]) -> Span[i32] from count:
    return values

fn main():
    println(0)
