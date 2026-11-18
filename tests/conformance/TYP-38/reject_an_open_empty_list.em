#$ test: compile-fail
#$ rules: TYP-38, TYP-23
## `[TYP-38]` - `[]` needs its element type from the context or from later uses in the function; left open it is
## `E2060`.

fn main():
    xs = []  #$ error[E2060]: cannot infer the element type of `xs`
    println(1)
