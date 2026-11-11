#$ test: compile-fail
#$ rules: CTL-7, BRW-2
#$ error[E3040]: `s`
# `[CTL-7]` — a local a `defer` block uses is not handed over below the `defer` line: its uses are
# uses at every exit of the block, so the hand-over is `E3040`.

fn consume(owns s: String):
    println(s)

fn main():
    s = String("a")
    defer:
        println(s)
    consume(owns s)
