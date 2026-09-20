#$ test: compile-pass
#$ rules: LT-1b, MAN-3
#$ warning[L3014]: return region is the intersection of 2 parameters

fn pick(a: Span[i32], b: Span[i32]) -> Span[i32]:
    return a
