#$ test: run-pass
#$ rules: FFI-39, FFI-39b
#$ warning[L3015]: `drive` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Layer.id` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Layer.total` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 4200 5 2
#$ stdout: 105 1 5
#$ stdout: 2505 25
# `[FFI-39]` — an Ember class derives from a C++ class its overlay declares `@ffi(trampoline,
# virtuals=[…])`: C++ calling a named virtual reaches the Ember override, and a class that
# overrides none behaves as the C++ class (`Plain`). `[FFI-39b]` — `super.init(…)` selects the
# C++ constructor by arity and argument types (`Layer(int)` for an `i32`, `Layer(double)` for an
# `f64`), and a class with no `init` is made by the default one.

import cpp "layers.hpp" with (overlay = "overlays/layers.em")

class Counting(cpp.Layer):
    ticks: i32 = 0

    fn init(self, id: i32):
        super.init(id)

    override fn on_update(self, dt: f32):
        self.ticks += 1

    override fn name(self) -> i32:
        return 42

class Plain(cpp.Layer):
    pass

class Scaled(cpp.Layer):
    fn init(self, scale: f64):
        super.init(scale)

fn main():
    counting = Counting(5)
    plain = Plain()
    scaled = Scaled(2.5)
    unsafe:  # SAFETY(ffi): calls on objects that live for each call
        println(cpp.drive(counting), counting.id(), counting.ticks)
        println(cpp.drive(plain), plain.id(), plain.total())
        println(cpp.drive(scaled), scaled.name())
