#$ test: run-pass
#$ rules: FFI-39, FFI-44
#$ profiles: debug, development, release
#$ stdout: 553 8
#$ stdout: 701 5
#$ stdout: 551
#$ stdout: 154
# `[FFI-39]`, `[FFI-44]` (ODR-221) — the trampoline passes on what every class of the program
# deriving from the C++ class overrides, however far down and in whichever module: a class of this
# module deriving from another module's class (named through its module, as an imported item, or
# through a type alias) overrides a virtual that one does not, and C++ calling it reaches it; so
# does a class deriving from the C++ class through a module's re-export of the header (`[FFI-30b]`).

import support.stage_kinds
from support.stage_kinds import Timed, Labelled

class Weighted(stage_kinds.Timed):
    override fn weight(self) -> i32:
        return 3

class Ticking(Timed):
    ticks: i32 = 0

    override fn tick(self, dt: i32):
        self.ticks += dt

    override fn label(self) -> i32:
        return 7

class Heavier(Labelled):
    override fn weight(self) -> i32:
        return 1

class Direct(stage_kinds.stages.Stage):
    override fn weight(self) -> i32:
        return 4

fn main():
    weighted = Weighted()
    ticking = Ticking()
    heavier = Heavier()
    direct = Direct()
    println(stage_kinds.run(weighted), ticking.ticks + 8)
    println(stage_kinds.run(ticking), ticking.ticks)
    println(stage_kinds.run(heavier))
    println(stage_kinds.run(direct))
