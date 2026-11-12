#$ test: compile-fail
#$ rules: ARN-5, TYP-15a, ARN-9
# `[TYP-15a]` and `[ARN-5]` allow an `ArenaMap` of views and an arena span of views, but the check that
# keeps such views no longer than the arena is not built by this compiler yet, so each is `E0900`,
# "not built yet", never accepted (D-721: it was `E2130`, whose meaning is a `const` that owns heap memory).

from std.collections import ArenaMap

@view
struct Name:
    text: StrView

fn main():
    arena = Arena.with_capacity(1024)
    _map = ArenaMap[Name, i32].with_capacity(arena, 1)    #$ error[E0900]: an `ArenaMap` with a view key or value is not built by this compiler yet
    _slots = arena.alloc_uninit[Name](2)    #$ error[E0900]: an uninitialized arena span of views is not built by this compiler yet
