#$ test: compile-fail
#$ rules: R12, R14
#$ help: fn show(owns it: owns dyn Iterator[Item = int] changes LOG) changes LOG:
## Verdict 46, R12 (rules.md lines 729-740): an adapter keeps the function it is given and carries its
## change list; a `dyn` type with no `changes` changes nothing, so an adapter whose steps change `LOG` is
## not made one (E2113). The message is rules.md section 3's line 1040, in this program's lines.

static mut LOG: Array[int] = []

fn show(owns it: owns dyn Iterator[Item = int]):
    for x in it:
        println(x)

fn main() changes LOG:
    xs = [1, 2]
    it = xs.iter().inspect(fn(x) changes LOG: LOG.push(x))
    show(it)    #$ error[E2113]: `it` runs a function that changes `LOG` (`inspect(fn(x) changes LOG: LOG.push(x))`, line 16), and `owns dyn Iterator[Item = int]` says its steps change nothing; say what they change: `fn show(owns it: owns dyn Iterator[Item = int] changes LOG) changes LOG:`
    println(LOG.len())
