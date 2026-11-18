#$ test: compile-fail
#$ rules: BLD-FFI-5a, FFI-31b
#$ profiles: debug
#$ help: write `thread_shared World` to hand it to C++
# `[BLD-FFI-5a]`, `[FFI-31b]` (ODR-162) — only a `thread_shared` value of a class crosses an export: C++
# may copy it on any thread, and only its counts are made for that. Any other object is `E5015`, whose
# help is to say `thread_shared`.

class World:
    people: i32 = 0

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_world"))
pub fn make_world() -> World:    #$ error[E5015]: the exported result is a `World`, which is not `thread_shared`
    return World()

# SAFETY: nothing else in the process defines this name
@unsafe(export("people_of"))
pub fn people_of(world: World) -> i32:    #$ error[E5015]: exported parameter `world` is a `World`
    return world.people

fn main():
    pass
