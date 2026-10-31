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
    fn get() -> int:
        return this.value

struct Words:
    value: StrView

extend Words implements TextSource:
    type Item = StrView
    fn get() -> StrView:
        return this.value

[P: needs Copy, S: needs TextSource, Q: needs Copy]
fn render(prefix: P, source: S, suffix: Q) -> String:
    item = text_value(source)
    return f"<{item.value}>|{item.value!r}|{item.value:>4}|{item.padded()}"

[P: needs Copy]
struct Renderer:
    marker: P

    [S: needs TextSource, Q: needs Copy]
    fn render(source: S, suffix: Q) -> String:
        item = text_value(source)
        return f"<{item.value}>|{item.value!r}|{item.value:>4}|{item.padded()}"

fn main():
    println(render(true, Numbers(7), 2.5))
    println(render(2.5, Words("word"), false))
    renderer = Renderer[StrView]("owner")
    println(renderer.render(Numbers(11), false))
