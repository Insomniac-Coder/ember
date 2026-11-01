## `std.coroutine` — gameplay sequencing on generators (`[CORO-11]`), with no
## support from the compiler beyond `gen fn` itself.
##
## A sequence is a `gen fn` yielding `Wait`s: what to wait for before it runs
## on. A `Scheduler` holds any number of them, of any functions, and resumes
## each once its wait is over, once per call to `update` (once per frame), and
## drops each as it finishes.
##
## ```ember,ignore
## from std.coroutine import Scheduler, Wait, wait
##
## gen fn blink(light: Light) -> Generator[Wait]:
##     for _ in 0..3:
##         light.on = not light.on
##         yield wait(0.5)
##     yield Wait.NextFrame
## ```

## What a sequence waits for before it is resumed.
pub enum Wait:
    ## The next `update`.
    NextFrame
    ## This many seconds of the time `update` is given.
    Seconds(float)
    ## This many `update`s.
    Frames(int)
    ## The first `update` at which this answers `true`.
    Until(fn() -> bool)

## Wait this many seconds.
pub fn wait(seconds: float) -> Wait:
    return Wait.Seconds(seconds)

## Wait this many frames: `wait_frames(1)` is `Wait.NextFrame`.
pub fn wait_frames(n: int) -> Wait:
    return Wait.Frames(n)

## Wait until `pred()` is `true`, asked once per frame.
[F: needs fn() -> bool]
pub fn wait_until(owns pred: F) -> Wait:
    return Wait.Until(owned fn() => pred())

## What a `Scheduler` resumes: any iterator of `Wait`s, a generator above all.
pub interface Sequence:
    fn resume() -> Option[Wait]

extend[G: needs Iterator[Item = Wait]] G implements Sequence:
    fn resume() -> Option[Wait]:
        return this.next()

## The sequences running, each with what it waits for. A sequence starts on
## the `update` after it is added.
pub struct Scheduler:
    running: Array[Box[dyn Sequence]]
    waiting: Array[Wait]

extend Scheduler:
    ## It allocates a new `Array`.
    pub static fn new() -> Scheduler:
        return Scheduler([], [])

    ## Run `sequence` from the next `update`.
    [G: needs Sequence]
    pub fn start(owns sequence: G):
        this.running.push(owns Box[dyn Sequence](sequence))
        this.waiting.push(owns Wait.NextFrame)

    ## How many sequences have not finished.
    pub fn len() -> int:
        return this.running.len()

    ## One frame, `dt` seconds long: resume each sequence whose wait is over,
    ## and drop each that finishes.
    ## It calls code whose effects are not declared, which may be code that allocates.
    pub fn update(dt: float):
        i = 0
        while i < this.running.len():
            if this.due(i, dt):
                match this.running[i].resume():
                    Some(next):
                        this.waiting[i] = next
                    None:
                        this.running.remove(i)
                        this.waiting.remove(i)
                        continue
            i += 1

    ## Whether sequence `i`'s wait is over, counting this frame against it.
    ## It calls code whose effects are not declared, which may be code that allocates.
    fn due(i: int, dt: float) -> bool:
        match this.waiting[i]:
            Wait.NextFrame:
                return true
            Wait.Seconds(left):
                if left <= dt:
                    return true
                this.waiting[i] = Wait.Seconds(left - dt)
                return false
            Wait.Frames(n):
                if n <= 1:
                    return true
                this.waiting[i] = Wait.Frames(n - 1)
                return false
            Wait.Until(ref mut pred):
                return pred()
