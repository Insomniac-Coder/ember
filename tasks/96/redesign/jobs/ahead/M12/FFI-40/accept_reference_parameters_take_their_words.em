# Clause FFI-40.1: "A `T&` parameter imports as `ref mut x: T`, `const T&` as a plain parameter `x:
# T`, and `T&&` as `owns x: T` where the type is movable"; "At a call from Ember the argument is
# handed over (`cache.add(owns splash)`), the thunk passes its address, and when the call returns the
# moved-from object is ended by `T`'s destructor"
#$ test: run-pass
#$ rules: FFI-40, FFI-40a, FN-2a
#$ profiles: debug, development, release
#$ stdout: 4
#$ stdout: 7
# `count` takes `const Splash&` (a plain argument), `grow` `Splash&` (`ref mut`), and `add` `Splash&&`
# (`owns`): `splash` ends at the hand-over. The overlay gives every member an entry.

import cpp "shelf.hpp" with (overlay = "overlays/shelf.em")

fn main():
    cache = cpp.Cache.new()
    splash = cpp.Splash(size=3)
    cache.count(splash)
    cache.grow(ref mut splash)
    println(splash.size)
    cache.add(owns splash)
    println(cache.total())
