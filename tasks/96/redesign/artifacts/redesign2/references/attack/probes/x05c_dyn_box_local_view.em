# Attack on A5's downcast / pool table plus A6's "a Box of views is a view": a type-erased box
# holding views of a local. Once boxed as `dyn AnyPool` the static type no longer says it holds views,
# so a later `cast keep[0] as Pool[str]` would read `text` after it is freed. Today: E3063.
interface AnyPool:
    fn count(self) -> int

struct Pool[T]:
    dense: Array[T]

extend[T] Pool[T] implements AnyPool:
    fn count(self) -> int:
        return self.dense.len()

fn fill(mut keep: Array[Box[dyn AnyPool]]):
    text: String = "short lived"
    keep.push(Box(Pool[str](dense=[text[0..5]])))

fn main():
    keep: Array[Box[dyn AnyPool]] = []
    fill(keep)
    println(keep[0].count())
