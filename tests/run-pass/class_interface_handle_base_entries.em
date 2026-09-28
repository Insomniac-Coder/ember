#$ test: run-pass
#$ rules: DSP-3, DSP-2, OBJ-2
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ stdout: 2
#$ assert-c: contains(em_itables_Base)
#$ assert-c: contains(em_itables_Child)
#$ assert-c: contains(->ti->vtable))->slot0)((struct em_obj_Base*)_0)

# Only a Base-typed handle is ever made a Named, so the Named entry is Base's.
# Child's list carries it as well (the inline lookup scans the object's own
# list), and the adapter calls the virtual `id` through the object's own
# class table, so Child's override runs (D-375).
interface Named:
    fn id(self) -> i32

open class Base implements Named:
    fn init(mut self):
        pass

    virtual fn id(self) -> i32:
        return 1

class Child(Base):
    fn init(mut self):
        super.init()

    override fn id(self) -> i32:
        return 2

fn show(value: Named):
    println(value.id())

fn main():
    base = Base()
    show(base)
    child = Child()
    as_base: Base = child
    show(as_base)
