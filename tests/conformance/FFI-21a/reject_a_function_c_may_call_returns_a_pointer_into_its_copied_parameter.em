#$ test: compile-fail
#$ rules: FFI-21a, FN-6
#$ profiles: debug
# `[FFI-21a]` — C passes `value` as a copy of its own, which ends when the call
# returns; a function whose value becomes an `extern "C" fn` does not hand back
# a pointer into it, directly or through a reference to one of its fields.

struct Pair:
    left: i32
    right: i32

fn address_of(value: Pair) -> *Pair:
    return ref_to_ptr(ref value)  #$ error[E5042]: `address_of` returns a pointer into `value`, which C passes as a copy

fn left_address(value: Pair) -> *i32:
    p = ref_to_ptr(ref value.left)
    return p  #$ error[E5042]: `left_address` returns a pointer into `value`, which C passes as a copy

fn give() -> extern "C" fn(Pair) -> *Pair:
    return address_of

fn give_left() -> extern "C" fn(Pair) -> *i32:
    return left_address

fn main():
    println(give() == give(), give_left() == give_left())
