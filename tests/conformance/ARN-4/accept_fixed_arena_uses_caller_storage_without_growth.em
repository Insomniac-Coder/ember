#$ test: run-pass
#$ rules: ARN-1, ARN-4, LT-4
# Arena.fixed borrows caller-provided bytes. Its allocation path aligns each
# value inside that span and never invokes the growing-arena constructor.

fn main():
    bytes: [u8; 24] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                       0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    fixed: FixedArena = Arena.fixed(bytes)
    first: ref mut i32 = fixed.alloc(30)
    second: ref mut i64 = fixed.alloc(12i64)
    first = 31
    println(first)
    println(second)
#$ stdout: 31
#$ 12
#$ assert-c: !contains("ember_arena_new")
#$ assert-c: contains("ember_fixed_arena_alloc_copy")
