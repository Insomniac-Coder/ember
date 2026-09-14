#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3
#$ stdout: 42
#$ stdout: 42

class Holder:
    value: i32

    fn show(self):
        println(self.value)

    fn init(mut self):
        self.value = 42
        self.show()

fn main():
    holder = Holder()
    println(holder.value)
