#$ test: run-pass
#$ rules: CLS-4, CLS-7, EXC-1
#$ profiles: debug, release, shipping
#$ assert-c: contains(ember_object_begin_write)
#$ assert-c: contains(ember_object_end_write)
#$ stdout: 3

class Child:
    value: i32

    fn bump(mut self):
        self.value = self.value + 1

class Holder:
    child: Child

fn main():
    child = Child(2)
    holder = Holder(child)
    holder.child.bump()
    println(holder.child.value)
