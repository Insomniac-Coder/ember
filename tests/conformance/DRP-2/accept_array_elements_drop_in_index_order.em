#$ test: run-pass
#$ rules: DRP-2, OWN-2
# "[DRP-2]: array elements in index order" — and the D-036 regression half:
# `a.push(R(1))` moves a temporary into the buffer, and the temporary must not
# be dropped behind it (`[OWN-3]`: the source is dead). Before the fix the
# temporary kept its statement-end drop and every element died twice (1 2 1 2).
# Output is the count here: any extra destruction prints.

struct R:
    pub n: i32

    fn drop(mut self):
        println(self.n)

fn main():
    a: Array[R] = Array[R]()
    a.push(R(1))
    a.push(R(2))
#$ stdout: 1
#$ 2
