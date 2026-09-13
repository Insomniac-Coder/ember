#$ test: compile-fail
#$ rules: SPN-1, SPN-3, BRW-1, BRW-2, BRW-5
#$ error[E3022]: `values` is already mutably borrowed

# Provenance must survive the composition of as_mut_span and split_at. The
# temporary parent view does not sever either child from the Array it borrows.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    parts = values.as_mut_span().split_at(1)
    left = parts.0
    values.push(3)
    println(left[0])
