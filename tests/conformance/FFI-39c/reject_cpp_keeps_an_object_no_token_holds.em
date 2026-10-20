#$ test: compile-fail
#$ rules: FFI-39c
# `[FFI-39c]` (ODR-156) — an object passed bare to a C++ parameter that keeps it would have nothing
# holding the `Retained` token that keeps it alive for C++: `E5060`.

import cpp "widgets.hpp" with (overlay = "overlays/widgets.em")

class Button(cpp.Widget):
    override fn draw(self) -> i32:
        return 7

    fn attach(self):
        cpp.add_widget(self)    #$ error[E5060]: C++ keeps this object, and nothing here holds the `Retained` token

fn main():
    Button().attach()
