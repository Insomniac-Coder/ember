#$ test: compile-fail
#$ rules: ARN-2, ARN-3
# Ordinary alloc cannot silently abandon a destructor. alloc_nodrop is the
# separate, explicit spelling when that is genuinely intended.

fn main():
    arena = Arena.with_capacity(64)
    values: Array[i32] = Array[i32]()
    stored = arena.alloc(values)  #$ error[E3090]: `Array[i32]` needs `drop` and cannot be allocated with `Arena.alloc`
