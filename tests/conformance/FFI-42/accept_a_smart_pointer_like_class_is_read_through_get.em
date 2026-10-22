#$ test: run-pass
#$ rules: FFI-42
#$ profiles: debug, development, release
#$ stdout: 3 4
#$ stdout: 9
# `[FFI-42]` (ODR-252) — a C++ class with `operator*` (and `operator->`) is read through: it imports
# `get()`, and `get_mut()` where the operator is not `const`; `->` is reached through `get()`, so a
# class with both imports one `get`. Without an overlay entry asserting them safe the calls are
# `unsafe` ([FFI-2]).

import cpp "handles.hpp"

fn main():
    unsafe:  # SAFETY(ffi): each handle owns its point for the whole of `main`
        h = cpp.Handle.new(3, 4)
        println(h.get().x, h.get().y)
        m = cpp.MutHandle.new(1, 2)
        m.get_mut().x = 9
        println(m.get_mut().x)
