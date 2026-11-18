#$ test: run-pass
#$ rules: CTL-7
#$ stdout: end
#$ second registered runs first
#$ defer sees 2
#$ drop 2
## `[CTL-7]` - a `defer` block uses the locals declared before it as they are when it runs (the change
## after the `defer` line shows), and they are not dropped before it: `t`'s `drop` runs after the block.

class T:
    n: int
    fn drop():
        println(f"drop {this.n}")

fn main():
    t = T(1)
    defer:
        println(f"defer sees {t.n}")
    t.n = 2
    defer:
        println("second registered runs first")
    println("end")
