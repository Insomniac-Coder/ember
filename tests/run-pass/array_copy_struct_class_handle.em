#$ test: run-pass
#$ rules: OWN-7, RC-1, RC-2, EXC-1
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2
#$ stdout: 7

class Thing:
    value: i32

@derive(Copy)
struct Wrapped:
    thing: Thing

fn populate(mut items: Array[Wrapped]):
    item = Wrapped(Thing(7))
    items.push(item)

fn main():
    items: Array[Wrapped] = Array[Wrapped]()
    populate(items)
    println(items[0].thing.value)
