# Clause EFF-26.3: "Putting a function value in a field, a list, a local or an iterator adapter
# changes nothing the function changes; the call that later runs it does."
#$ test: run-pass
#$ rules: EFF-26, CLO-17, BRW-18
#$ stdout: start
#$ stdout: 4
#$ stdout: 7
#$ stdout: 9
# Inside a walk of `LOG`, `log_it` is put in a local, a field, a list and an adapter, and nothing
# runs it; after the walk each is run.

static mut LOG: Array[String] = []

fn log_it():
    LOG.push(owns String("it"))

fn quiet():
    pass

struct Hook:
    run: fn() = quiet

fn tally(n: int) -> int:
    LOG.push(owns String("seen"))
    return n

fn main():
    LOG.push(owns String("start"))
    h: fn() = quiet
    k = Hook()
    hs: Array[fn()] = []
    nums = [3, 4]
    for line in LOG:
        println(line)
        h = log_it
        k.run = log_it
        hs.push(owns log_it)
        _kept = nums.iter().map(owns fn(n) => tally(n))
    h()
    k.run()
    hs[0]()
    println(LOG.len())
    println(nums.iter().map(owns fn(n) => tally(n)).sum())
    println(LOG.len() + 3)
