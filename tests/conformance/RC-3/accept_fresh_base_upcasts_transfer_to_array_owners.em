#$ test: run-pass
#$ rules: RC-3, RC-1, CLS-4, DSP-2
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ stdout: 7
#$ stdout: drop
#$ stdout: cleared

open class Base:
    fn value(self) -> i32:
        return 7

    fn drop(mut self):
        println("drop")

class Child(Base):
    pass

fn main():
    values: Array[Base] = Array[Base]()
    values.push(Child())
    println(values[0].value())
    values.clear()
    println("cleared")
