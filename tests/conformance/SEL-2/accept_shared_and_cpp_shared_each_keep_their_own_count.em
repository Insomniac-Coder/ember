#$ test: compile-pass
#$ rules: SEL-2
# `[SEL-2]`: a program may hold both kinds side by side; each stays its own type. A `CppShared`
# clones to a `CppShared` (C++'s count) and a `Shared` to a `Shared` (Ember's), and the object a
# `CppShared` shares is reached through a reference, never by converting the pointer. (`Shared` is
# `Copy`, `[HEAP-6]`: a copy retains.)
from std.ffi import CppShared

fn read_through_cpp(pointer: ref CppShared[i32]) -> i32:
    copy: CppShared[i32] = pointer.clone()
    return copy.get()

fn read_through_ember(pointer: Shared[i32]) -> i32:
    copy: Shared[i32] = pointer
    return copy.get()

fn main():
    shared: Shared[i32] = Shared(5)
    println(read_through_ember(shared))
