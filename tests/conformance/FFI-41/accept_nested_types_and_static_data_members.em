#$ test: run-pass
#$ rules: FFI-41, FFI-3
#$ profiles: debug, development, release
#$ warning[W5034]: `hidden` is not imported from `space.hpp`: it is in an anonymous namespace
#$ stdout: 3 7 32 1
#$ stdout: 2 0.5 61
#$ stdout: wall
# `[FFI-41]` — a nested type sits under its outer type's name: Ember has no nested types, so
# `Grid::Cell` is `Grid_Cell`, `Grid::Fill` `Grid_Fill` and `Grid::Index` `Grid_Index`. A static
# data member that is a constant is its class's constant (`Grid.width`); any other, and a variable
# of a namespace, is reached through the address a thunk gives (`Grid.made() -> *mut c_int`),
# since its own symbol is mangled and Ember links no mangled symbol (`[FFI-3]`).

import cpp "space.hpp"

fn main():
    area: cpp.engine.Grid_Index = cpp.engine.Grid.width * cpp.engine.Grid.height
    fill = cpp.engine.Grid_Fill.Wall
    unsafe:  # SAFETY(ffi): calls on values, and reads and writes of C++'s variables
        corner: cpp.engine.Grid_Cell = cpp.engine.Grid.corner()
        println(corner.row, corner.column, area, cpp.engine.Grid.made().read())
        _ = cpp.engine.Grid.corner()
        cpp.engine.frames().write(cpp.engine.frames().read() + 1)
        println(cpp.engine.Grid.made().read(), cpp.engine.Grid.scale().read(), cpp.engine.frames().read())
    match fill:
        cpp.engine.Grid_Fill.Wall:
            println("wall")
        _:
            println("empty")
