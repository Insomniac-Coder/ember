#$ test: compile-fail
#$ rules: FFI-48
#$ warning[W5002]: `pooled` is not imported from `pooled.hpp`: its result: `std::vector<int, Pool<int>>`: an allocator-parameterised container
#$ warning[W5002]: `pooled_total` is not imported from `pooled.hpp`: parameter `values`: `std::vector<int, Pool<int>>`: an allocator-parameterised container
# `[FFI-48]` — an allocator-parameterised container is unsupported: the allocator is part of the
# type and opaque, so a function taking or returning one is not imported, and naming it is `E5034`,
# naming the row; a `std::vector` with the standard allocator imports as a `CppVector` (`[FFI-17a]`).

from std.ffi import CppVector
import cpp "pooled.hpp"

fn main():
    unsafe:  # SAFETY(ffi): never run
        _ = cpp.plain(3)
        _ = cpp.pooled(3)    #$ error[E5034]: an allocator-parameterised container
        _ = cpp.pooled_total    #$ error[E5034]: parameter `values`
