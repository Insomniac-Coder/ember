#$ test: run-pass
#$ rules: ARN-2, ARN-3
# alloc_nodrop is the explicit acknowledgement that the arena will reclaim the
# bytes without running the stored value's destructor.

struct Resource:
    value: i32

extend Resource:
    fn drop(mut self):
        println(self.value)

fn main():
    arena = Arena.with_capacity(64)
    resource: ref mut Resource = arena.alloc_nodrop(Resource(7))
    println(resource.value + 2)
#$ stdout: 9
