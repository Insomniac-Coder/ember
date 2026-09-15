#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-3, CLS-6
#$ profiles: debug, release, shipping
#$ assert-c: contains(em_default_value())
#$ stdout: 3
#$ stdout: 7
#$ stdout: 3

fn default_value() -> i32:
    return 3

class Defaults:
    value: i32 = default_value()

    fn init(mut self):
        println(self.value)
        self.value = self.value + 4

class Memberwise:
    value: i32 = default_value()

fn main():
    value = Defaults()
    println(value.value)
    memberwise = Memberwise()
    println(memberwise.value)
