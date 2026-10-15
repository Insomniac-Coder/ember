#$ test: compile-fail
#$ rules: DIA-4, EFF-6, EFF-5
#$ error[E4001]: render -> layout -> measure -> a new `Array`
#$ error[E4040]: tick -> step -> fail
# `[DIA-4]` — a contract error prints the whole call chain, every function between the
# contracted one and the offending operation, whichever contract it breaks.

fn measure() -> int:
    v = [1, 2, 3]
    return v.len()

fn layout() -> int:
    return measure()

@noalloc
fn render() -> int:
    return layout()

fn fail():
    panic("stop")

fn step():
    fail()

@nopanic(explicit)
fn tick():
    step()

fn main():
    println(render())
    tick()
