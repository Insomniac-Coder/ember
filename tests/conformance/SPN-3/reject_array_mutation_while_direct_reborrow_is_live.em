#$ test: compile-fail
#$ rules: SPN-1, SPN-3, BRW-1, BRW-2
#$ error[E3022]: `values` is already mutably borrowed

# A reborrow made directly from a view-producing expression retains the
# original Array provenance; the temporary parent does not sever the loan.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    child = values.as_mut_span().reborrow()
    values.push(2)
    println(child[0])
