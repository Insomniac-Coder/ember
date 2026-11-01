#$ test: run-pass
#$ rules: OBJ-2, DSP-3, IFC-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c-count: contains("ember_itable_lookup") == 2

# A mutable holder may replace its interface field between calls, so its two
# lookups must remain distinct rather than reuse the immutable-field cache.
interface Render:
    fn render() -> i32

class Pixel implements Render:
    value: i32

    fn render() -> i32:
        return this.value

class Holder:
    value: Render

fn render_after_replace(ref mut holder: Holder) -> i32:
    first = holder.value.render()
    holder.value = Pixel(22)
    return first + holder.value.render()

fn main():
    pixel = Pixel(20)
    holder = Holder(pixel)
    println(render_after_replace(ref mut holder))
