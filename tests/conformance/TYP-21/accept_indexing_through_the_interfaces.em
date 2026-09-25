#$ test: run-pass
#$ rules: TYP-21, STD-17
#$ profiles: debug, release, shipping
#$ stdout: 3.0
#$ [1.0, 9.5, 3.0, 4.5]
#$ set 3
#$ ['x']
# `[TYP-21]` — `a[i]` reads through `Index.index`, `a[i] op= v` writes in
# place through `IndexMut.index_mut`, and `a[i] = v` calls
# `IndexSet.index_set` where the type has it (`[STD-17]`).

struct Grid:
    w: int
    cells: Array[f32]

extend Grid implements Index[(int, int)], IndexMut[(int, int)]:
    type Output = f32

    fn index(self, at: (int, int)) -> ref f32:
        return ref self.cells[at.0 * self.w + at.1]

    fn index_mut(mut self, at: (int, int)) -> ref mut f32:
        return ref mut self.cells[at.0 * self.w + at.1]

struct Log:
    lines: Array[String]

extend Log implements IndexSet[int, String]:
    fn index_set(mut self, i: int, owned v: String):
        println(f"set {i}")
        self.lines.push(v)

fn main():
    g = Grid(w=2, cells=[1.0, 2.0, 3.0, 4.0])
    println(g[(1, 0)])
    g[(0, 1)] = 9.5
    g[(1, 1)] += 0.5
    println(g.cells)
    log = Log(lines=[])
    log[3] = String.from("x")
    println(log.lines)
