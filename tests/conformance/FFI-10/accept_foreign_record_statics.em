#$ test: parse-pass
#$ rules: FFI-10, FFI-8, STA-1

@derive(Copy)
struct ForeignPair:
    left: i32
    right: i32

@derive(Copy)
struct ForeignInner:
    value: i32

@derive(Copy)
struct ForeignOuter:
    inner: ForeignInner
    other: i32

unsafe extern "C":
    static mut shared_pair: ForeignPair
    @ffi(immutable)
    static frozen_pair: ForeignPair
    safe fn read_pair_sum() -> i32
    fn bump_pair_left() -> i32
    static mut nested_pair: ForeignOuter
    @ffi(link_name="nested_pair")
    static mut aliased_nested: ForeignOuter
    @ffi(immutable)
    static frozen_nested: ForeignOuter
    safe fn read_nested_total() -> i32
    fn bump_nested_other() -> i32

fn main():
    println(frozen_pair.left + frozen_pair.right)
    println(frozen_nested.inner.value)
    unsafe:
        println(shared_pair.left + shared_pair.right)
        shared_pair = ForeignPair(left = 8, right = 9)
        println(read_pair_sum())
        shared_pair.left = 12
        println(read_pair_sum())
        shared_pair.right += bump_pair_left()
        println(read_pair_sum())
        nested_pair.inner.value = 10
        println(read_nested_total())
        nested_pair.inner.value += bump_nested_other()
        println(read_nested_total())
        aliased_nested.inner.value = 30
        println(read_nested_total())
