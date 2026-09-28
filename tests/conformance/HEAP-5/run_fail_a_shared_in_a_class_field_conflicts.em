#$ test: run-fail
#$ rules: HEAP-5, HEAP-6, EXC-1
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: overlapping access
# D-374 — a `Shared[T]` stored in a class field is checked like one in a
# local: reading its value through the field while a mutable borrow is taken
# through another handle panics. Before the fix it began no access at all.

struct Counter:
    value: i32

class Holder:
    shared: Shared[Counter]

    fn init(mut self):
        self.shared = Shared(Counter(1))

fn main():
    h = Holder()
    alias = h.shared
    reader = h.shared.get()
    writer = alias.get_mut()
    println(reader.value + writer.value)
