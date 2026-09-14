#$ test: compile-fail
#$ rules: CLS-1, CLS-2
#$ error[E1010]: class construction for `OwnedField` is not implemented yet in this phase

class OwnedField:
    values: Array[i32]

    fn drop(mut self):
        pass

fn main():
    value = OwnedField(Array[i32]())
    println(1)
