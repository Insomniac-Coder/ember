#$ test: run-pass
#$ rules: TYP-23, TYP-18
#$ profiles: debug, development, release
#$ stdout: 5
#$ 42
#$ 7
#$ 11
#$ 6 8
#$ assert-c: contains("int32_t em_2idG2c2a47066b4fc729")
#$ assert-c: contains("int32_t em_4pickG2c2a47066b4fc729")
#$ assert-c: contains("em_6HolderG2c2a47066b4fc729")
#$ assert-c: !contains("em_6HolderG73498f1dbd18d400")
#$ assert-c: contains("sizeof(int32_t), _Alignof(int32_t)")
# The expected type flows into a generic call and a generic constructor, and a
# literal argument waits for the typed arguments: nothing here becomes `int`.
# (`[MNG-1]`: an instance's symbol ends with `G` and its arguments' hash, `2c2a47066b4fc729` for
# `[i32]` and `73498f1dbd18d400` for `[i64]`.)

struct Holder[T]:
    value: T

fn id[T](x: T) -> T:
    return x

fn pick[T](a: T, b: T) -> T:
    return b

fn main():
    small: i32 = id(5)
    println(small)
    holder: Holder[i32] = Holder(42)
    println(holder.value)
    seven: i32 = 7
    println(pick(seven, 7))
    arena = Arena.with_capacity(16)
    slot: ref mut i32 = arena.alloc(11)
    println(slot)
    again: i32 = id(6)
    println(again, pick(seven, 8))
