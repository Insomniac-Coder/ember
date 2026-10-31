#$ test: compile-fail
#$ rules: TYP-16, IFC-1, CLS-4
#$ profiles: debug, development, release
#$ error[E2110]: override of a method that is not virtual

[T]
open class Base:
    fn tick():
        pass

    fn init():
        pass

[T]
class Child(Base[T]):
    fn init():
        super.init()

extend[T] Child[T]:
    override fn tick():
        pass

fn main():
    bool_child = Child[bool]()
    int_child = Child[i32]()
    bool_child.tick()
    int_child.tick()
