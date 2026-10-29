#$ test: run-pass
#$ rules: FFI-17a
#$ warning[L3015]: `first_of` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Grid.raw_cells` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 6 0 0
#$ stdout: 1 4 10
#$ stdout: 9 9 7
# `[FFI-17a]` (ODR-200) — a `std::span<T, N>` of fixed extent crosses as `ref [T; N]` (`ref mut
# [T; N]` for a span of `T`, not `const T`): the type states the length, so nothing is checked at
# the call. As a result it borrows what the overlay's entry names; with no entry the call is
# `unsafe` and a result is its elements' address.

import cpp "fixed.hpp" with (project = "views", overlay = "overlays/fixed.em")

fn main():
    three: [i32; 3] = [1, 2, 3]
    two: [i32; 2] = [5, 6]
    cpp.zero2(ref mut two)
    println(cpp.sum3(ref three), two[0], two[1])
    grid = cpp.Grid.new()
    cells = grid.get().cells()
    println(cells[0], cells[3], cells[0] + cells[1] + cells[2] + cells[3])
    edited = cpp.Grid.new()
    writable = edited.get_mut().cells_mut()
    writable[0] = 9
    first: i32 = 0
    raw: *i32 = null[*i32]()
    unsafe:  # SAFETY(ffi): `two` lives for the call; the grid lives while its cells are read
        first = cpp.first_of(ref two)
        raw = edited.get().raw_cells()
    unsafe:  # SAFETY: four `i32`s at `raw`, in the grid that lives
        println(raw.read(), edited.get().cells()[0], first + 7)
