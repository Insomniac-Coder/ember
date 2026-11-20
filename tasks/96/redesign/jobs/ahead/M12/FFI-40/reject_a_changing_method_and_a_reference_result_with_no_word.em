# Clause FFI-40.2: "every other method may change it, so it is not called through a name that only
# reads the object (`E3023`)"; "without one the method stays unsafe to call, its result a `*mut T` or
# `*T` (...), and a call of it outside `unsafe:` is `E3100`"
#$ test: compile-fail
#$ rules: FFI-40, FFI-40a, FFI-11, UNS-1
# `clear` has no `@ffi(const)`, so it changes its object, and `show`'s `cache` only reads (code
# only). `parent`'s `Node&` result has no lifetime word in its entry: the spec's message, with this
# file's line number. The test only compiles.

import cpp "shelf.hpp" with (overlay = "overlays/shelf.em")

fn show(cache: cpp.Cache):
    println(cache.total())
    cache.clear()    #$ error[E3023]

fn main():
    child = cpp.Node.new()
    _p = child.parent()    #$ error[E3100]: `child.parent()` returns a C++ reference whose target no contract names; put line 18 in an `unsafe:` block with a `# SAFETY:` note, or give `parent` a lifetime word in the overlay: `fn parent() -> borrowed one from(this)`
    show(cpp.Cache.new())
