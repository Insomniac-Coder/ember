#$ test: run-pass
#$ rules: ARN-3, ARN-11, ARN-12, TST-23
# A source `Default` implementation must not replace the built-in proof that
# all-zero is valid. `[ARN-3]` makes Zeroable precedence observable semantics.

from std.core import Default

extend i32 implements Default:
    fn default() -> i32:
        return 77

fn main():
    arena = Arena.with_capacity(16)
    values = arena.alloc_array[i32](1)
    println(values[0])
#$ stdout: 0
