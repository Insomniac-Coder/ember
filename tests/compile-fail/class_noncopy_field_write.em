#$ test: compile-fail
#$ rules: CLS-1, CLS-7, EXC-1
#$ error[E1010]: mutable class-field access requires a `mut self` class method

class Holder:
    values: Array[i32]

fn main():
    values: Array[i32] = Array()
    holder = Holder(values)
    replacement: Array[i32] = Array()
    holder.values = replacement
