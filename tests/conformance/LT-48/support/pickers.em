## `[LT-48]` — the kinds and the checked method of `run_fail_a_kind_made_in_another_module_is_checked.em`.

pub interface Picker:
    fn pick(other: Array[int]) -> Span[int]

pub struct Mine implements Picker:
    pub items: Array[int]
    fn pick(other: Array[int]) -> Span[int]:
        return this.items

pub struct Yours implements Picker:
    pub items: Array[int]
    fn pick(other: Array[int]) -> Span[int]:
        return other

pub struct Shower:
    pub n: int
    pub fn show(p: runtime dyn Picker):
        xs = [1, 2]
        r = p.pick(xs)
        xs.push(owns 3)
        println(r)
