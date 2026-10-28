#$ test: run-pass
#$ rules: BRW-12, FN-4, BRW-16
#$ stdout: start
#$ two
#$ 3
## design rules: R2, R5, R8
## R2, R5, R8 (c): the nearest program that is allowed: `swap_in()` runs after the loop over `LOG`; the
## old `Res` ends at the second `=` and the new one when `swap_in` returns, each `drop` pushing once
## (verdict 57), and no first line names it (verdict 86).

static mut LOG: Array[String] = []

class Res:
    name: String

    fn drop():
        LOG.push(String("ended"))

fn swap_in():
    a = Res(name=String("one"))
    a = Res(name=String("two"))
    println(a.name)

fn main():
    LOG.push(String("start"))
    for line in LOG:
        println(line)
    swap_in()
    println(LOG.len())
