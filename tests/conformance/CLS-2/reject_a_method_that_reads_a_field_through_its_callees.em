# Clause CLS-2.1: "`init` may call a method of its object once every field that method's body
# (...) reads is set", the fields its body reads worked out through its callees (`[EFF-24]`)
#$ test: compile-fail
#$ rules: CLS-2, EFF-24
# `prepare` reads `this.assets` only through `count`, and lets `this` out only through `enrol`. No
# text in the spec for this program: code only.

static mut ROSTER: Array[weakref Game] = []

class Game:
    renderer: int
    assets: Array[int]
    loaded: int

    fn count() -> int:
        return this.assets.len()

    fn prepare() -> int:
        return this.count()

    fn enrol():
        ROSTER.push(weakref this)

    fn register():
        this.enrol()

    fn init(r: int):
        this.renderer = r
        this.loaded = this.prepare()    #$ error[E2100]
        this.register()    #$ error[E2100]
        this.assets = [1]

fn main():
    _g = Game(1)
