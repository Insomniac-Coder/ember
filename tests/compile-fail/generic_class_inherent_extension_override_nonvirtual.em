#$ test: compile-fail
#$ rules: TYP-16, IFC-1, CLS-4
#$ profiles: debug, release, shipping
#$ error[E2110]: override of a method that is not virtual

open class Base[T]:
    fn tick(self):
        pass

    fn init(mut self):
        pass

class Child[T](Base[T]):
    fn init(mut self):
        super.init()

extend[T] Child[T]:
    override fn tick(self):
        pass

fn main():
    bool_child = Child[bool]()
    int_child = Child[i32]()
    bool_child.tick()
    int_child.tick()
