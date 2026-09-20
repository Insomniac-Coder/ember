#$ test: run-pass
#$ rules: TYP-16, CLS-4, DSP-4, OBJ-2, DRP-6
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ stdout: 7
#$ stdout: 0
#$ stdout: 1
#$ assert-c: contains(ember_downcast)

open class Base[T]:
    value: i32
    marker: T

    fn init(mut self, value: i32, marker: T):
        self.value = value
        self.marker = marker

class Child[T](Base[T]):
    fn init(mut self, value: i32, marker: T):
        super.init(value, marker)

class Other[T](Base[T]):
    fn init(mut self, value: i32, marker: T):
        super.init(value, marker)

fn main():
    child = Child[bool](7, true)
    same = child
    base: Base[bool] = child
    if child is same:
        println(1)
    else:
        println(0)
    hit: Option[Child[bool]] = base as? Child[bool]
    match hit:
        Some(value):
            println(value.value)
        None:
            println(0)
    miss: Option[Other[bool]] = base as? Other[bool]
    match miss:
        Some(_):
            println(1)
        None:
            println(0)
    forced: Child[bool] = base as! Child[bool]
    if forced is child:
        println(1)
    else:
        println(0)
