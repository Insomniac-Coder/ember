#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, CTL-3
#$ stdout: 42

class Holder:
    value: i32

    fn init():
        this.value = 42
        for i in 0..0:
            println(this.value)

fn main():
    holder = Holder()
    println(holder.value)
