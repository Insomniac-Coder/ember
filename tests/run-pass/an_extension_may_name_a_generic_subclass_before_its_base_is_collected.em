#$ test: run-pass
#$ rules: CLS-4, TYP-16
#$ stdout: true
# A generic extension's associated type names an instance of a generic class whose base is an
# ordinary class (`Option[Store[T]]`). The instance is made while extensions are collected, before
# any class's own methods are: its `override` was checked against a base with no methods yet and
# refused (`E2110`). It is now checked once every class's methods are recorded.
open class AnyStore:
    n: int

    fn init(mut self):
        self.n = 0

    virtual fn remove(mut self, i: int) -> bool:
        return false

class Store[T](AnyStore):
    items: Array[T]

    fn init(mut self):
        self.items = []
        super.init()

    override fn remove(mut self, i: int) -> bool:
        return true

interface Held:
    type State
    fn state() -> State

struct Marker[T]:
    pass

extend[T] Marker[T] implements Held:
    type State = Option[Store[T]]

    fn state() -> Option[Store[T]]:
        return Some(Store[T]())

fn main():
    match Marker[int].state():
        Some(store):
            base: AnyStore = store
            println(base.remove(0))
        None:
            println(false)
