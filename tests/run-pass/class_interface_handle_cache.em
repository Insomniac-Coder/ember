#$ test: run-pass
#$ rules: DSP-3, OBJ-2
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c-count: contains(ember_itable_lookup) == 1

# `[DSP-3]` permits a per-handle hidden lookup cache: both calls below use the
# unchanged one-word `Render` handle and therefore share its interface table.
interface Render:
    fn render(self) -> i32

class Pixel implements Render:
    value: i32

    fn render(self) -> i32:
        return self.value

fn render_twice(value: Render) -> i32:
    return value.render() + value.render()

fn main():
    pixel = Pixel(21)
    println(render_twice(pixel))
