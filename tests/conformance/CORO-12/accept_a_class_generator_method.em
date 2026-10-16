#$ test: run-pass
#$ rules: CORO-12, CORO-1
#$ profiles: debug, release
#$ stdout: 1.5 3.0 4.5
#$ 4.5
#$ 1
# `[CORO-12]` — a `gen fn` method of a class takes `self`, and the frame retains the handle:
# each access to the object inside is checked on its own, and the object outlives every
# handle but the frame's.

class Door:
    open_angle: float = 0.0

    gen fn swing_open(self, steps: int) -> Generator[float]:
        for _ in 0..steps:
            self.open_angle += 1.5
            yield self.open_angle

fn make() -> Door:
    return Door()

fn main():
    door = make()
    angles = [a for a in door.swing_open(3)]
    println(angles[0], angles[1], angles[2])
    println(door.open_angle)
    g = make().swing_open(1)
    for _ in g:
        println(1)
