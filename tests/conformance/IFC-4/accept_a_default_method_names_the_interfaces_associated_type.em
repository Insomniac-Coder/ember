#$ test: run-pass
#$ rules: IFC-4, CLO-6
#$ profiles: debug, development, release
#$ stdout: [6, 8] | 3 | ok
# D-381 — a default method names its interface's associated type by its bare
# name, as the declaration does: in a callable parameter's type
# (`keep: fn(Item) -> bool`), a local's annotation (`Array[Item]`) and its
# result. Each implementing type's copy reads it as that type's own: `int`
# for `Evens`, `String` for `Words`.

interface Source:
    type Item
    fn get(i: int) -> Item
    fn size() -> int

    fn kept(keep: fn(Item) -> bool) -> Array[Item]:
        out: Array[Item] = []
        for i in 0..this.size():
            x = this.get(i)
            if keep(x):
                out.push(x)
        return out

struct Evens:
    n: int

extend Evens implements Source:
    type Item = int

    fn get(i: int) -> int:
        return 2 * i

    fn size() -> int:
        return this.n

struct Words:
    words: Array[String]

extend Words implements Source:
    type Item = String

    fn get(i: int) -> String:
        return this.words[i].clone()

    fn size() -> int:
        return this.words.len()

fn main():
    e = Evens(5)
    w = Words([String.from("ok"), String.from("no"), String.from("yes")])
    kept = w.kept(fn(s) => s.len() == 2)
    println(e.kept(fn(x) => x > 5), "|", e.kept(fn(x) => x % 4 == 0).len(), "|", kept[0])
