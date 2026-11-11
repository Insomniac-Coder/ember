#$ test: compile-fail
#$ rules: OWN-6, OWN-3
#$ error[E3040]: `resource` was handed over on line 15 and is used on line 16; to keep both, hand over a copy of your own: `copy of resource`

import std.mem

struct Resource:
    pub value: i32

    fn drop():
        pass

fn main():
    resource = Resource(7)
    mem.forget(resource)
    println(resource.value)
