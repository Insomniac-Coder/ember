#$ test: run-pass
#$ rules: CLS-7, EXC-17
#$ stdout: 1
# A plain class method may write a Copy field without a whole-object access.

class ReadOnly:
    value: i32

    fn bad(self):
        self.value = 1

fn main():
    value = ReadOnly(0)
    value.bad()
    println(value.value)
