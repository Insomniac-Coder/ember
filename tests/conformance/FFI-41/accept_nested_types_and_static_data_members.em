#$ test: run-pass
#$ rules: FFI-41, FFI-3
#$ warning[L3015]: `Grid.corner` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ warning[W5034]: `hidden` is not imported from `space.hpp`: it is in an anonymous namespace
#$ stdout: 3 7 32 1
#$ stdout: 12 0.5 61
#$ stdout: wall
# `[FFI-41]` — a nested type sits under its outer type's name: Ember has no nested types, so
# `Grid::Cell` is `Grid_Cell`, `Grid::Fill` `Grid_Fill` and `Grid::Index` `Grid_Index`. A static
# data member that is a constant is its class's constant (`Grid.width`); any other, and a variable
# of a namespace, reads and writes as a global (`Grid.made += 10`, ODR-161), each use going through
# a thunk that gives its address, since its own symbol is mangled and Ember links no mangled
# symbol (`[FFI-3]`). A `const` one is read outside `unsafe` (`Grid.scale`).

import cpp "space.hpp"

fn main():
    area: cpp.engine.Grid_Index = cpp.engine.Grid.width * cpp.engine.Grid.height
    fill = cpp.engine.Grid_Fill.Wall
    unsafe:  # SAFETY(ffi): calls on values, and reads and writes of C++'s variables
        corner: cpp.engine.Grid_Cell = cpp.engine.Grid.corner()
        println(corner.row, corner.column, area, cpp.engine.Grid.made)
        _ = cpp.engine.Grid.corner()
        cpp.engine.Grid.made += 10
        cpp.engine.frames = cpp.engine.frames + 1
        println(cpp.engine.Grid.made, cpp.engine.Grid.scale, cpp.engine.frames)
    match fill:
        cpp.engine.Grid_Fill.Wall:
            println("wall")
        _:
            println("empty")
