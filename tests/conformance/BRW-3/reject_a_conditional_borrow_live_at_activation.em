#$ test: compile-fail
#$ rules: BRW-3, BCK-5, TYP-5

fn append(mut values: Array[int], value: ref int):
    values.push(value)

fn main():
    values: Array[int] = [7]
    append(values, values[0] if values.len() > 0 else values[0])    #$ error[E3021]: borrowed here and mutably borrowed elsewhere
    #$ error[E3021]: borrowed here and mutably borrowed elsewhere
