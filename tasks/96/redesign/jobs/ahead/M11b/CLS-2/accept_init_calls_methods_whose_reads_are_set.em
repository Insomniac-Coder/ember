# Clause CLS-2.1: "after `this.renderer = r`, `this.assets = this.load_assets()` is allowed when
# `load_assets` reads only `this.renderer`"; "in any other class every override is the package's
# own, and the check reads their bodies"
#$ test: run-pass
#$ rules: CLS-2, DSP-5
#$ stdout: 2
#$ stdout: 7
# The spec's allowed call, and a `virtual` method called from the `init` of an `open` class that is
# not `pub`: every override is this package's, and none reads a field not yet set.

class Game:
    renderer: int
    assets: Array[int]

    fn init(r: int):
        this.renderer = r
        this.assets = this.load_assets()

    fn load_assets() -> Array[int]:
        return [this.renderer, this.renderer]

open class Base:
    tag: int
    size: int

    fn init():
        this.tag = 7
        this.size = this.measure()

    virtual fn measure() -> int:
        return this.tag

class Big(Base):
    fn init():
        super.init()

    override fn measure() -> int:
        return this.tag * 1

fn main():
    g = Game(3)
    println(g.assets.len())
    b = Big()
    println(b.size)
