#$ test: run-pass
#$ rules: FFI-39, FFI-44, FFI-39b
#$ profiles: debug, development, release
#$ stdout: 4201 5 0
#$ stdout: 159 5
#$ stdout: 151
#$ stdout: 90 3
# `[FFI-39]`, `[FFI-44]` (ODR-221) — an Ember class derives from a C++ class with no overlay
# entry: the trampoline the compiler writes passes on the virtuals the program's classes override
# (`override fn`), read from the classes, and C++ calling one reaches the Ember override. A class
# that overrides none, or not this one, behaves as the C++ class (the virtual reaches the C++
# implementation). A C++ class with a pure virtual is an abstract class: each Ember class deriving
# from it implements it.

import cpp "stages.hpp"

class Counting(cpp.Stage):
    ticks: i32 = 0

    fn init(self, id: i32):
        super.init(id)

    override fn tick(self, dt: i32):
        self.ticks += dt

    override fn label(self) -> i32:
        return 42

class Heavy(cpp.Stage):
    override fn weight(self) -> i32:
        return 9

class Plain(cpp.Stage):
    pass

class Square(cpp.Shape):
    side: i32 = 3

    override fn area(self) -> i32:
        return self.side * self.side

fn main():
    counting = Counting(7)
    heavy = Heavy()
    plain = Plain()
    square = Square()
    unsafe:  # SAFETY(ffi): calls on objects that live for each call
        println(cpp.run(counting), counting.ticks, counting.total())
        println(cpp.run(heavy), heavy.total())
        println(cpp.run(plain))
        println(cpp.measure(square), square.side)
