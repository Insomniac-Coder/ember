# A3, the one hazard left when two names may be one element: a view into the element's own heap
# data (its name's characters) kept across a write that replaces that data through the other name.
struct Unit:
    name: String
    hp: int

fn rename(mut a: Unit, mut b: Unit):
    old = a.name.as_str()
    b.name = "renamed"          # if a and b are one Unit, this frees the buffer `old` views
    println(old)

fn main():
    us = [Unit(name="ann", hp=1), Unit(name="bob", hp=2)]
    rename(us[0], us[1])
