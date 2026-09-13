#$ test: compile-fail
#$ rules: OWN-6, OWN-3
#$ error[E3040]: `resource` has been moved out of

import std.mem

struct Resource:
    pub value: i32

    fn drop(mut self):
        pass

fn main():
    resource = Resource(7)
    mem.forget(resource)
    println(resource.value)
