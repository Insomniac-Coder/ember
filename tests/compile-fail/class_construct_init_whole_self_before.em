#$ test: compile-fail
#$ rules: CLS-2
#$ error[E2100]: `self` is used before all class fields are initialized

class Holder:
    value: i32

    fn show(self):
        println(self.value)

    fn init(mut self):
        self.show()
        self.value = 42

fn main():
    holder = Holder()
