#$ test: run-pass
#$ rules: CLS-1, EXC-1, FN-2a
#$ profiles: debug, release, shipping
#$ assert-c: contains(ember_field_begin_write)
#$ assert-c: contains(ember_field_end_write)
#$ stdout: 1

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
    println(holder.items[0].value)
