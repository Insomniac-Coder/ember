# replaced_in_call.em: a name for a list field, then a call that MAY replace that field.
struct Bag:
    items: Array[int] = []
    dirty: bool = false

    fn reset(mut self):
        if self.dirty:
            self.items = []

fn main():
    b = Bag()
    inv = b.items
    b.reset()
    inv.push(1)
    println(len(b.items))
