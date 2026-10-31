#$ test: compile-fail
#$ rules: IFC-4, TYP-17, MOD-4, STD-9, LEX-19
# Nearby parameter slots have Display and Debug, but OpaqueSource.Item has
# neither. Importing its wrapper must not transfer those unrelated bounds.

from support.formatting_projection_values import OpaqueSource, opaque_value

struct Numbers:
    value: int

extend Numbers implements OpaqueSource:
    type Item = int
    fn get() -> int:
        return this.value

[P: needs (Display, Debug), S: needs OpaqueSource, Q: needs (Display, Debug)]
fn render_free(prefix: P, source: S, suffix: Q):
    item = opaque_value(source)
    println(item.value)    #$ error[E2040]

[P: needs (Display, Debug)]
struct Renderer:
    marker: P

    [S: needs OpaqueSource, Q: needs (Display, Debug)]
    fn render(source: S, suffix: Q):
        item = opaque_value(source)
        println(item.value)    #$ error[E2040]
        println(f"{item.value:>4}")    #$ error[E2040]

fn main():
    render_free("prefix", Numbers(7), "suffix")
    renderer = Renderer[StrView]("owner")
    renderer.render(Numbers(11), "suffix")
