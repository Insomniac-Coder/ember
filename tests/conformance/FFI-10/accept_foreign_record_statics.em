#$ test: parse-pass
#$ rules: FFI-10, FFI-8, STA-1

@derive(Copy)
struct ForeignPair:
    left: i32
    right: i32

unsafe extern "C":
    static mut shared_pair: ForeignPair
    @ffi(immutable)
    static frozen_pair: ForeignPair
    safe fn read_pair_sum() -> i32
    fn bump_pair_left() -> i32

fn main():
    println(frozen_pair.left + frozen_pair.right)
    unsafe:
        println(shared_pair.left + shared_pair.right)
        shared_pair = ForeignPair(left = 8, right = 9)
        println(read_pair_sum())
        shared_pair.left = 12
        println(read_pair_sum())
        shared_pair.right += bump_pair_left()
        println(read_pair_sum())
