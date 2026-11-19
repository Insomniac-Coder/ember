#$ test: run-pass
#$ rules: TYP-14
#$ stdout: 4
#$ 9
#$ 5
#$ 9
#$ 7
#$ 12
## `[TYP-14]` - a `ref T` and an `owns T` are read through wherever a `T` is wanted: a field access, a method
## call, an operand, and an argument to a plain parameter.

struct Mesh:
    n: int

    fn count() -> int:
        return this.n

fn plain(m: Mesh) -> int:
    return m.n

fn via_ref(ref m: Mesh) -> int:
    return m.n + m.count() + plain(m)

fn main():
    mesh = Mesh(4)
    r = ref mesh
    println(r.n)                      # a field through a `ref`
    println(r.count() + 5)            # a method through a `ref`
    held: owns Mesh = owns Mesh(5)
    println(held.n)                  # a field through an `owns`
    println(held.count() + held.n - 1)
    println(plain(held) + 2)         # an argument to a plain parameter
    println(via_ref(r))
