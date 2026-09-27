#$ test: parse-pass
#$ rules: FFI-10, FFI-9

struct ForeignPair:
    left: i32
    right: i32

struct ForeignWide:
    a: i64
    b: i64
    c: i64

unsafe extern "C":
    safe fn add_pairs(owned a: ForeignPair, owned b: ForeignPair) -> ForeignPair
    safe fn mutate_pair_copy(pair: ForeignPair) -> i32
    safe fn swap_pair(pair: ForeignPair) -> ForeignPair
    @ffi(link_name="foreign_pair_weighted")
    fn weighted_pair(pair: ForeignPair, weight: i32) -> i32
    @ffi(param(data, borrowed, count(n)))
    safe fn pair_and_values(pair: ForeignPair, data: Span[i32], n: u8) -> i32
    fn get_pair_callback() -> extern "C" fn(ForeignPair) -> i32
    safe fn transform_wide(value: ForeignWide) -> ForeignWide

fn main():
    result = add_pairs(ForeignPair(left = 3, right = 4), ForeignPair(left = 5, right = 6))
    println(result.left)
    println(result.right)
    original = ForeignPair(left = 3, right = 4)
    println(mutate_pair_copy(original))
    println(original.left)
    swapped = swap_pair(original)
    println(swapped.left)
    println(swapped.right)
    unsafe:
        println(weighted_pair(original, 3))
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    println(pair_and_values(original, values.as_span()))
    unsafe:
        callback = get_pair_callback()
        println(callback(original))
    wide = ForeignWide(a = 10, b = 20, c = 30)
    changed = transform_wide(wide)
    println(changed.a)
    println(changed.b)
    println(changed.c)
    println(wide.a)
