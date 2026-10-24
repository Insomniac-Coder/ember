# Attack on field effects rule 3 ("by class and field"): an inherited field reached through a base
# handle and a derived handle that may be the same object.
open class Script:
    items: Array[int] = []

class Door(Script):
    pass

fn copy_into(s: Script, d: Door):
    for x in s.items:
        d.items.push(x)              # the same list when d is s

fn main():
    d = Door()
    d.items.push(1)
    copy_into(d, d)
