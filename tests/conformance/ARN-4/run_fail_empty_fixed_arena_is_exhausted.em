#$ test: run-fail
#$ rules: ARN-4
# An empty caller buffer is a valid fixed arena with no remaining capacity.
# Allocation reports exhaustion; it is not misclassified as a malformed arena.

fn main():
    bytes: [u8; 0] = []
    fixed: FixedArena = Arena.fixed(bytes)
    value: ref mut i32 = fixed.alloc(1)
    println(value)
#$ panics: fixed arena exhausted
