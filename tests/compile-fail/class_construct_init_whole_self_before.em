#$ test: compile-fail
#$ rules: CLS-2
#$ error[E2100]: `this` is used before all class fields are initialized

class Holder:
    value: i32

    fn show():
        println(this.value)

    fn init():
        this.show()
        this.value = 42

fn main():
    holder = Holder()
