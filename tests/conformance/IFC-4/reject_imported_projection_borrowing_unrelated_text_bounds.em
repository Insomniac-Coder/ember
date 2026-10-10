#$ test: compile-fail
#$ rules: IFC-4, TYP-17, MOD-4, STD-9, LEX-19
# Nearby parameter slots have Display and Debug, but OpaqueSource.Item has
# neither. Importing its wrapper must not transfer those unrelated bounds.

from support.formatting_projection_values import OpaqueSource, opaque_value

struct Numbers:
    value: int

extend Numbers implements OpaqueSource:
    type Item = int
    fn get(self) -> int:
        return self.value

fn render_free[P: Display + Debug, S: OpaqueSource, Q: Display + Debug](prefix: P, source: S, suffix: Q):
    item = opaque_value(source)
    println(item.value)    #$ error[E2040]

struct Renderer[P: Display + Debug]:
    marker: P

    fn render[S: OpaqueSource, Q: Display + Debug](self, source: S, suffix: Q):
        item = opaque_value(source)
        println(item.value)    #$ error[E2040]
        println(f"{item.value:>4}")    #$ error[E2040]

fn main():
    render_free("prefix", Numbers(7), "suffix")
    renderer = Renderer[str]("owner")
    renderer.render(Numbers(11), "suffix")
