#$ test: run-pass
#$ rules: IFC-4, CLO-6
#$ profiles: debug, release, shipping
#$ stdout: [6, 8] | 3 | ok
# D-381 — a default method names its interface's associated type by its bare
# name, as the declaration does: in a callable parameter's type
# (`keep: fn(Item) -> bool`), a local's annotation (`Array[Item]`) and its
# result. Each implementing type's copy reads it as that type's own: `int`
# for `Evens`, `String` for `Words`.

interface Source:
    type Item
    fn get(self, i: int) -> Item
    fn size(self) -> int

    fn kept(self, keep: fn(Item) -> bool) -> Array[Item]:
        out: Array[Item] = []
        for i in 0..self.size():
            x = self.get(i)
            if keep(x):
                out.push(x)
        return out

struct Evens:
    n: int

extend Evens implements Source:
    type Item = int

    fn get(self, i: int) -> int:
        return 2 * i

    fn size(self) -> int:
        return self.n

struct Words:
    words: Array[String]

extend Words implements Source:
    type Item = String

    fn get(self, i: int) -> String:
        return self.words[i].clone()

    fn size(self) -> int:
        return self.words.len()

fn main():
    e = Evens(5)
    w = Words([String.from("ok"), String.from("no"), String.from("yes")])
    kept = w.kept(fn(s) => s.len() == 2)
    println(e.kept(fn(x) => x > 5), "|", e.kept(fn(x) => x % 4 == 0).len(), "|", kept[0])
