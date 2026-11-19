#$ test: compile-fail
#$ rules: CLS-2
# `[CLS-2]` — a method called from `init` reads a field that is set only later.

class Game:
    renderer: int
    assets: int

    fn init(r: int):
        this.renderer = r
        this.assets = this.load_assets()    #$ error[E2100]

    fn load_assets() -> int:
        return this.assets + 1

fn main():
    pass
