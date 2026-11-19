#$ test: compile-fail
#$ rules: BRW-3, BCK-5, TYP-5

fn append(ref mut values: Array[int], from: Span[int]):
    values.push(owns from[0])

fn main():
    values: Array[int] = [7]
    append(ref mut values, values.as_span() if values.len() > 0 else values.as_span())    #$ error[E3021]: borrowed here and mutably borrowed elsewhere
    #$ error[E3021]: borrowed here and mutably borrowed elsewhere
