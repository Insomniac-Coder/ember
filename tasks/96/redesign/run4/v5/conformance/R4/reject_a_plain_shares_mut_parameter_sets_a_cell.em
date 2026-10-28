#$ test: compile-fail
#$ rules: BRW-15
#$ help: fn tick(ref mut s: shares mut Clock):
## design rules: R4, R5
## R4, verdict 32: the type after the colon says what the caller's value is, never what the function
## may do with it; a plain `s: shares mut Clock` only reads, so it does not set the clock's `Cell`
## (AN-20).

struct Clock:
    frame: Cell[int]

fn tick(s: shares mut Clock):
    s.frame.set(s.frame.get() + 1)    #$ error[E3023]: `tick` sets `s.frame`, but `s` only reads the caller's clock (`shares mut Clock` says what the caller's clock is, not what `tick` may do with it); write `fn tick(ref mut s: shares mut Clock):` and call it `tick(ref mut clock)`

fn main():
    clock = shares mut Clock(frame=Cell(0))
    tick(clock)
    println(clock.frame.get())
