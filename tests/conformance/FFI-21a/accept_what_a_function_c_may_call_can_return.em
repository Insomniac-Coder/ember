#$ test: run-pass
#$ rules: FFI-21a, FN-6, FN-1
#$ profiles: debug, release, shipping
#$ stdout: 1 true true true
# `[FFI-21a]` — a function whose value becomes an `extern "C" fn` may return a
# pointer C passed (taken as a pointer), a value read through a parameter C
# copies, and a pointer such a parameter holds: none points into the copy. A
# function only Ember calls may return a pointer to what its caller lent it
# (`[FN-1]`: the borrowed parameter is the caller's value).

struct Pair:
    left: i32
    right: i32

struct Holder:
    target: *Pair
    count: i32

fn same(value: *Pair) -> *Pair:
    return value

fn left_of(value: Pair) -> i32:
    return value.left

fn target_of(value: Holder) -> *Pair:
    return value.target

fn address_of(value: Pair) -> *Pair:
    return ref_to_ptr(ref value)

fn give_same() -> extern "C" fn(*Pair) -> *Pair:
    return same

fn give_left() -> extern "C" fn(Pair) -> i32:
    return left_of

fn give_target() -> extern "C" fn(Holder) -> *Pair:
    return target_of

fn main():
    pair = Pair(left = 1, right = 2)
    here = ref_to_ptr(ref pair)
    holder = Holder(target = here, count = 3)
    native: fn(Pair) -> *Pair = address_of
    same_c = give_same()
    left_c = give_left()
    target_c = give_target()
    println(left_c(pair), same_c(here) == here, target_c(holder) == here, native(pair) == here)
