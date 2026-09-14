#$ test: compile-fail
#$ rules: CLS-1, EXC-1, FN-2a
#$ error[E1010]: mutable class-field access requires a `mut self` class method

class Inner:
    value: i32

class Holder:
    items: Array[Inner]

fn increment(mut value: i32):
    value = value + 1

fn main():
    items: Array[Inner] = Array()
    items.push(Inner(0))
    holder = Holder(items)
    increment(holder.items[0].value)
