# Clause CLS-2.1: "In a class that is `pub` and `open` or `abstract`, an override of a `virtual`
# method may be any dependant's (...): its `init` calls a `virtual` method only after every field is
# set"; "`init` may call a method of its object once every field that method's body (every
# override's, for a `virtual` method) reads is set"; "a method that lets `this` out is called only
# after every field is set"
#$ test: compile-fail
#$ rules: CLS-2, BLD-16, EFF-24
# The spec's three messages, with this file's line numbers (the existing CLS-2 tests claim these
# cases' code only).

static mut ROSTER: Array[weakref Player] = []

pub open class Script:
    name: String

    fn init(owns name: String):
        this.on_make()    #$ error[E2100]: `init` calls `on_make()` (line 17) before `name` is set (line 18), and `Script` is `pub` and `open`, so any package may override `on_make` with a body that reads `name` or keeps a link to the object; set every field first: move line 17 below line 18
        this.name = name

    virtual fn on_make():
        pass

class Game:
    renderer: int
    assets: Array[int]
    loaded: int

    fn init(r: int):
        this.renderer = r
        this.loaded = this.load_assets()    #$ error[E2100]: `init` calls `load_assets()` (line 30), which reads `this.assets` (its line 34), and `assets` is set only on line 31; give it a value first: write `this.assets = []` above line 30
        this.assets = [1]

    fn load_assets() -> int:
        return this.assets.len()

class Player:
    name: String

    fn enrol():
        ROSTER.push(weakref this)

    fn init(owns name: String):
        this.enrol()    #$ error[E2100]: `enrol` keeps a link to the player (`ROSTER.push(weakref this)`, its line 40), and `name` is set only on line 44; set every field first: move line 43 below line 44
        this.name = name

fn main():
    _s = Script(String("s"))
    _g = Game(1)
    _p = Player(String("p"))
