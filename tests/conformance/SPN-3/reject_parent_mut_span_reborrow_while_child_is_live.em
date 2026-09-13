#$ test: compile-fail
#$ rules: SPN-3, BRW-1, BRW-2
#$ error[E3022]: `parent` is already mutably borrowed

# A reborrow preserves the parent value but reserves its exclusive access
# until the child view's final use.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    parent = values.as_mut_span()
    child = parent.reborrow()
    _other = parent.reborrow()
    println(child[0])
