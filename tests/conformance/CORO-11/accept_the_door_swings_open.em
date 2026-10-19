#$ test: run-pass
#$ rules: CORO-11, CORO-12
#$ profiles: debug, development
#$ stdout: 0.0
#$ 0.0
#$ 1.5
#$ 3.0
#$ 3.0 1
#$ blinks 3, running 0
#$ waited until 2
# `[CORO-11]` — `std.coroutine` sequences gameplay on generators: the specification's door, which
# waits half a second, then opens a little each frame, under a `Scheduler` that resumes it once
# per frame and drops it when it ends; and a second sequence of another function beside it.

from std.coroutine import Scheduler, Wait, wait, wait_frames, wait_until

class Door:
    open_angle: float = 0.0

    gen fn swing_open(self) -> Generator[Wait]:
        yield wait(0.5)
        for _ in 0..2:
            self.open_angle += 1.5
            yield Wait.NextFrame

class Counter:
    blinks: int = 0

gen fn blink(c: Counter) -> Generator[Wait]:
    for _ in 0..3:
        c.blinks += 1
        yield wait_frames(2)

gen fn waiting_for(c: Counter) -> Generator[Wait]:
    yield wait_until(owned fn() => c.blinks == 0)
    c.blinks = 2

fn main():
    door = Door()
    c = Counter()
    s = Scheduler.new()
    s.start(door.swing_open())
    s.start(blink(c))
    s.update(0.25)
    println(door.open_angle)
    s.update(0.25)
    println(door.open_angle)
    s.update(0.25)
    println(door.open_angle)
    s.update(0.25)
    println(door.open_angle)
    s.update(0.25)
    println(door.open_angle, s.len())
    s.update(0.25)
    s.update(0.25)
    println(f"blinks {c.blinks}, running {s.len()}")
    t = Scheduler.new()
    t.start(waiting_for(c))
    t.update(0.1)
    c.blinks = 0
    t.update(0.1)
    t.update(0.1)
    println("waited until", c.blinks)
