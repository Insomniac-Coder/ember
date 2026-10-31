#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3
#$ stdout: 42
#$ stdout: 42

class Holder:
    value: i32

    fn show():
        println(this.value)

    fn init():
        this.value = 42
        this.show()

fn main():
    holder = Holder()
    println(holder.value)
