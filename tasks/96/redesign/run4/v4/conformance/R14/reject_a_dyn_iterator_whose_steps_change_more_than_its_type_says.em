#$ test: compile-fail
#$ rules: R12, R14
## R12 (verdict 46): "a value whose methods change more than its `dyn` type says is refused (E2113),
## and a `dyn` type with no `changes` changes nothing" (section 3's E2113 message, in this program's
## lines).

static mut LOG: Array[int] = []

fn show(owns it: owns dyn Iterator[Item = int]):
    for x in it:
        println(x)

fn main() changes LOG:
    xs = [1, 2]
    show(xs.iter().inspect(fn(x) changes LOG: LOG.push(x)))    #$ error[E2113]: `it` runs a function that changes `LOG` (`inspect(fn(x) changes LOG: LOG.push(x))`, line 15), and `owns dyn Iterator[Item = int]` says its steps change nothing; say what they change: `fn show(owns it: owns dyn Iterator[Item = int] changes LOG) changes LOG:`
    println(LOG.len())
