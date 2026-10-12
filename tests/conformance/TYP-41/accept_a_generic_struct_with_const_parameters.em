#$ test: run-pass
#$ rules: TYP-41, TYP-16
#$ stdout: 6 6
#$ stdout: 2
## `[TYP-41]` — a generic struct's `const` parameters: `Matrix[2, 3]` is a
## type, its field's length is computed from them, its methods read them, and
## `Matrix[N, N]` solves `N`.

struct Matrix[const R: int, const C: int]:
    cells: [f32; R * C]

    fn size(self) -> int:
        return R * C

fn trace[const N: int](m: Matrix[N, N]) -> int:
    return N

fn main():
    m: Matrix[2, 3] = Matrix[2, 3]([0.0; 6])
    println(len(m.cells), m.size())
    square = Matrix[2, 2]([1.0; 4])
    println(trace(square))
