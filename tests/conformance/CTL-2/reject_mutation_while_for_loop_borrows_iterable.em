#$ test: compile-fail
#$ rules: CTL-2, DIA-7, DIA-7a, DIA-10, DIA-13, PHIL-8a

# A source `for` loop retains its iterable loan for the whole loop. This must
# be the dedicated B2 diagnostic, not generic shared/mutable overlap (D-070).
fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    values.push(owns 2)
    for value in values:
        values.push(owns 3) #$ error[E3020]: `values.push(owns 3)` (line 11) adds to `values`, which this loop walks (line 10); collect what it adds in a list of your own: write `added: Array[i32] = []` before the loop, `added.push(owns 3)` in place of line 11, and `values.extend(owns added)` after the loop
        println(value)
