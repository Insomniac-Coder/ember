#$ test: compile-fail
#$ rules: BLD-FFI-5a, FFI-31b
#$ profiles: debug
#$ help: mark `World` `@sync` to hand it to C++
# `[BLD-FFI-5a]`, `[FFI-31b]` (ODR-162) — only a `@sync` class's object crosses an export: C++ may
# copy its handle on any thread, and only a `@sync` class's counts are atomic. Any other class is
# `E5015`, whose help is to mark it `@sync`.

class World:
    people: i32 = 0

@export("make_world")
pub fn make_world() -> World:    #$ error[E5015]: the exported result is a `World`, whose class is not `@sync`
    return World()

@export("people_of")
pub fn people_of(world: World) -> i32:    #$ error[E5015]: exported parameter `world` is a `World`
    return world.people

fn main():
    pass
