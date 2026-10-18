#$ test: run-pass
#$ rules: ARN-1, ARN-4, ARN-6, TYP-36
#$ profiles: debug, release
#$ stdout: ok
#$ ok
#$ ok
# `[ARN-1]`, `[TYP-36]` (D-679) — every arena allocates the unit value `()` as any other: the C copied
# from `&0`, the address of a constant, which no C compiler accepts. A zero-sized value copies nothing.

fn main():
    arena = Arena.with_capacity(64)
    _a = arena.alloc(())
    println("ok")
    with scope = arena.scope():
        _b = scope.alloc(())
        println("ok")
    bytes: Array[u8] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    fixed: FixedArena = Arena.fixed(bytes)
    _c = fixed.alloc(())
    println("ok")
