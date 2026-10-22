#$ test: compile-fail
#$ rules: SEL-2, WK-14
# `[SEL-2]`: Ember's `Shared`/`Weak` and C++'s `std::shared_ptr` held as `CppShared` never convert
# into each other, in either direction and in every position that converts a value (an argument,
# a return, an annotated local, an `as` cast), because they keep different counts: each is `E5065`,
# not a plain type mismatch.
from std.ffi import CppShared

fn give_to_cpp(pointer: CppShared[i32]):
    pass

fn keep_in_ember(pointer: Shared[i32]):
    pass

fn back_to_ember(pointer: CppShared[i32]) -> Shared[i32]:
    return pointer    #$ error[E5065]: Ember's and C++'s reference counts are different counts

fn main():
    shared: Shared[i32] = Shared(5)
    give_to_cpp(shared)    #$ error[E5065]: `Shared[i32]` does not convert to `std.ffi.CppShared[i32]`
    weak = Weak(shared)
    give_to_cpp(weak)    #$ error[E5065]: `Weak[Shared[i32]]` does not convert
    held: CppShared[i32] = Shared(6)    #$ error[E5065]: counted by Ember
    cast = shared as CppShared[i32]    #$ error[E5065]: both would free it
