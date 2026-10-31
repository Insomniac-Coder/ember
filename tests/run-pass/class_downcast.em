#$ test: run-pass
#$ rules: DSP-4, OBJ-2, ERR-1
#$ profiles: debug, development, release
#$ stdout: 7
#$ stdout: 0
#$ stdout: 1

open class Base:
    value: i32

    fn init(value: i32):
        this.value = value

class Child(Base):
    fn init(value: i32):
        super.init(value)

class Other(Base):
    fn init(value: i32):
        super.init(value)

fn main():
    child = Child(7)
    base: Base = child
    hit: Option[Child] = cast base if Child
    match hit:
        Some(value):
            println(value.value)
        None:
            println(0)
    miss: Option[Other] = cast base if Other
    match miss:
        Some(_):
            println(0)
        None:
            println(0)
    forced: Child = cast base as Child
    if forced is child:
        println(1)
    else:
        println(0)
