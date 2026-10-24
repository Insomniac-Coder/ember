# Attack on A5 3.5 ("the return type says ref, so the return need not") together with A22 6.6
# ("a value returned by a call is a place until the end of its statement"): with both, `return
# self.total()` under `-> ref int` would lend a temporary that dies at the return. Today's spelling
# of what 3.5 would produce is `return ref self.total()`; it must stay an error.
struct Acc:
    xs: Array[int]

    fn total(self) -> int:
        s = 0
        for x in self.xs:
            s += x
        return s

    fn total_ref(self) -> ref int:
        return ref self.total()

fn main():
    a = Acc(xs=[1, 2])
    println(a.total_ref())
