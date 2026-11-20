# Clause FFI-39.1: "An `override fn` matching no virtual method of the base, by name and first line —
# each parameter's word and type exactly those the import gives the virtual method's (`[FFI-40]`),
# and its result; an upcast or a widening counts for nothing (...) — is `E5056`, whose message gives
# each virtual method of that name as the `override fn` line to write"
#$ test: compile-fail
#$ rules: FFI-39, FFI-40, FFI-57
# The spec's message: `on(const Event&)` imports as `on(e: gui.ui.Event)`, and a `KeyEvent` is no
# match. The test only compiles.

import cpp "handlers.hpp" as gui

class Keys(gui.ui.Handler):
    override fn on(e: gui.ui.KeyEvent):    #$ error[E5056]: `on(e: gui.ui.KeyEvent)` overrides no virtual method of `ui.Handler`: it declares `on(e: gui.ui.Event)` and `on(timer_id: c_int)`; write `override fn on(e: gui.ui.Event):` or `override fn on(timer_id: c_int):`
        println(1)

fn main():
    _keys = Keys()
