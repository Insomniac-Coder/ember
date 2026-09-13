#$ test: compile-fail
#$ rules: CTL-2, DIA-7, DIA-7a, DIA-10, DIA-13, PHIL-8a

# A source `for` loop retains its iterable loan for the whole loop. This must
# be the dedicated B2 diagnostic, not generic shared/mutable overlap (D-070).
fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    for value in values:
        values.push(3) #$ error[E3020]: cannot mutate `values` while it is borrowed by this loop
        println(value)
