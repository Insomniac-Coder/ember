#$ test: parse-pass
#$ rules: FFI-9

struct ForeignPair:
    left: i32
    right: i32

pub extern "C" fn exported_pair_score(pair: ForeignPair) -> i32:
    return pair.left * 2 + pair.right

pub extern "C" fn exported_pair_mix(left: ForeignPair, scale: i32, right: ForeignPair) -> ForeignPair:
    return ForeignPair(left = left.left * scale + right.left, right = left.right * scale + right.right)

unsafe extern "C":
    safe fn call_exported_pair_score() -> i32
    safe fn call_exported_pair_mix() -> i32
    fn call_pair_callback(callback: extern "C" fn(ForeignPair) -> i32) -> i32

fn main():
    pair = ForeignPair(left = 3, right = 4)
    println(exported_pair_score(pair))
    println(pair.left)
    println(call_exported_pair_score())
    other = ForeignPair(left = 4, right = 5)
    mixed = exported_pair_mix(pair, 2, other)
    println(mixed.left)
    println(mixed.right)
    println(call_exported_pair_mix())
    callback: extern "C" fn(ForeignPair) -> i32 = exported_pair_score
    println(callback(pair))
    unsafe:
        println(call_pair_callback(callback))
