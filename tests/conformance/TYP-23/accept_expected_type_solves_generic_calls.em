#$ test: run-pass
#$ rules: TYP-23, TYP-18
#$ profiles: debug, release, shipping
#$ stdout: 5
#$ 42
#$ 7
#$ 11
#$ assert-c: contains("int32_t em_id__")
#$ assert-c: contains("int32_t em_pick__")
#$ assert-c: contains("em_Holder_i32")
#$ assert-c: !contains("em_Holder_i64")
#$ assert-c: contains("sizeof(int32_t), _Alignof(int32_t)")
# The expected type flows into a generic call and a generic constructor, and a
# literal argument waits for the typed arguments: nothing here becomes `int`.

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
