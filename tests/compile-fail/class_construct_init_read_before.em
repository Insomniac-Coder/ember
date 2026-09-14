#$ test: compile-fail
#$ rules: CLS-1, CLS-2
#$ error[E2100]: field `y` is read before it is initialized

class Bad:
    x: i32
    y: i32

    fn init(mut self, x: i32):
        self.x = self.y
        self.y = x

fn main():
    value = Bad(1)
    println(value.x)
