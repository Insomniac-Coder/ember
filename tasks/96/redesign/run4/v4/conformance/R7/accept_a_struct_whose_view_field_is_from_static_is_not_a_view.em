#$ test: run-pass
#$ rules: R7
#$ stdout: positive
## R7: `from static` on a struct's field does not make the struct a view, so it may be returned from
## anywhere and kept in a class field with no `from`.

struct Tag:
    label: StrView from static

fn make_tag(n: int) -> Tag:
    if n > 0:
        return Tag(label="positive")
    return Tag(label="not positive")

class Item:
    tag: Tag

fn main():
    it = Item(tag=make_tag(3))
    println(it.tag.label)
