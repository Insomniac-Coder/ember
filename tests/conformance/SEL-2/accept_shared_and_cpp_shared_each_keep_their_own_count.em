#$ test: compile-pass
#$ rules: SEL-2
# `[SEL-2]`: a program may hold both kinds side by side; each stays its own type. A `CppShared`
# clones to a `CppShared` (C++'s count) and a `shares` owner makes another with `shares` (Ember's), and
# the object a `CppShared` shares is reached through a reference, never by converting the pointer.
# (Another owner is made only by a line that says so, `[HEAP-6]`.)
from std.ffi import CppShared

fn read_through_cpp(ref pointer: CppShared[i32]) -> i32:
    copy: CppShared[i32] = copy of pointer
    return copy.get()

fn read_through_ember(pointer: shares mut i32) -> i32:
    copy: shares i32 = shares pointer
    return copy

fn main():
    shared: shares mut i32 = shares mut 5
    println(read_through_ember(shared))
