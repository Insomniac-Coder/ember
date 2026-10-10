#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-18, MOD-4, LEX-19, STD-9
#$ stdout: <7>|7|   7|   7
#$ stdout: <word>|'word'|word|word
#$ stdout: <11>|11|  11|  11
# A projection carried by an imported generic struct keeps its text bounds in
# functions and methods whose other parameters put it in different slots.

from support.formatting_projection_values import TextSource, text_value

struct Numbers:
    value: int

extend Numbers implements TextSource:
    type Item = int
    fn get(self) -> int:
        return self.value

struct Words:
    value: str

extend Words implements TextSource:
    type Item = str
    fn get(self) -> str:
        return self.value

fn render[P: Copy, S: TextSource, Q: Copy](prefix: P, source: S, suffix: Q) -> String:
    item = text_value(source)
    return f"<{item.value}>|{item.value!r}|{item.value:>4}|{item.padded()}"

struct Renderer[P: Copy]:
    marker: P

    fn render[S: TextSource, Q: Copy](self, source: S, suffix: Q) -> String:
        item = text_value(source)
        return f"<{item.value}>|{item.value!r}|{item.value:>4}|{item.padded()}"

fn main():
    println(render(true, Numbers(7), 2.5))
    println(render(2.5, Words("word"), false))
    renderer = Renderer[str]("owner")
    println(renderer.render(Numbers(11), false))
