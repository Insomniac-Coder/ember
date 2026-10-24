# Attack on field effects (2.1): std's own CppVector. `push_back` writes no field of `self`; it
# calls a C++ thunk that may reallocate the vector's elements. Today `mut self` on push_back is what
# refuses this program. Under "mut is inferred from the body", the body reshapes nothing of `self`.
from std.ffi import CppVector, c_int

fn grow_while_viewing(mut v: CppVector[c_int]):
    s = v.span()            # borrows v: a view of the C++ vector's elements
    v.push_back(7)          # C++ push_back: may reallocate the elements
    println(s[0])           # reads the old element block

fn main():
    pass
