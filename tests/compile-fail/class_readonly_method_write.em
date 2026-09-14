#$ test: compile-fail
#$ rules: CLS-7, EXC-1
#$ error[E3023]: cannot mutate borrowed parameter `self`

class ReadOnly:
    value: i32

    fn bad(self):
        self.value = 1

fn main():
    value = ReadOnly(0)
    value.bad()
