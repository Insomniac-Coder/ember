#$ test: run-pass
#$ rules: CLS-4, CLS-7, EXC-1
#$ profiles: debug, release, shipping
#$ assert-c: contains(em_Base_bump)
#$ assert-c-count: contains("ember_object_begin_write") == 1
#$ assert-c-count: contains("ember_object_end_write") == 1
#$ assert-c: !contains("_access_child")
#$ stdout: 7

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

    fn bump(mut self, amount: i32):
        self.value = self.value + amount

class Derived(Base):
    fn init(mut self, value: i32):
        super.init(value)

class Holder:
    child: Derived

fn main():
    child = Derived(2)
    holder = Holder(child)
    holder.child.bump(5)
    println(holder.child.value)
