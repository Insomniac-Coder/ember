#$ test: compile-fail
#$ rules: R12, R14, R5
## R12 (verdict 46): "a function that drives it names the change (`fn show(owns it: owns dyn
## Iterator[Item = int] changes LOG) changes LOG:`)"; without it, E2105 (R5).

static mut LOG: Array[int] = []

fn show(owns it: owns dyn Iterator[Item = int] changes LOG):    #$ error[E2105]: `show` drives `it`, whose steps change `LOG` (its type says `owns dyn Iterator[Item = int] changes LOG`); add it to `show`'s first line: `fn show(owns it: owns dyn Iterator[Item = int] changes LOG) changes LOG:`
    for x in it:
        println(x)

fn main() changes LOG:
    xs = [1, 2]
    show(xs.iter().inspect(fn(x) changes LOG: LOG.push(x)))
    println(LOG.len())
