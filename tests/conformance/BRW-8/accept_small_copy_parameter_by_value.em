#$ test: run-pass
#$ rules: BRW-8
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(int32_t em_add_by_value(int32_t _1, int32_t _2))
#$ assert-c: !contains(const int32_t* em_add_by_value)

# `[BRW-8]` permits the ABI to replace a shared borrow of a small `Copy`
# value with a copy. Ember chooses that representation for `i32`; this is
# intentionally observable only in generated C, never to the source program.
fn add_by_value(left: i32, right: i32) -> i32:
    return left + right

fn main():
    value: i32 = 20
    println(add_by_value(value, 22))
