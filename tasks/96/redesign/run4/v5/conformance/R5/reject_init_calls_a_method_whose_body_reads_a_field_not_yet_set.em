#$ test: compile-fail
#$ rules: CLS-2
#$ help: give it a value first: write `this.assets = []` above line 18
## design rules: R5
## R5, row 7: `init` may call a method of the object it builds once every field that method's body reads
## is set, worked out from the body (verdict 86): `load_assets` reads `this.assets`, which only the line
## that calls it sets (E2100, `V5/rules.md` section 3's message in this program's lines). The nearest
## program that is allowed is `accept_init_calls_a_method_once_the_fields_its_body_reads_are_set`.

class Renderer:
    width: int

class Game:
    renderer: Renderer
    assets: Array[String]

    fn init(width: int):
        this.renderer = Renderer(width=width)
        this.assets = this.load_assets()    #$ error[E2100]: `init` calls `load_assets()` (line 18), which reads `this.assets` (line 21), and `assets` is set only by line 18 itself; give it a value first: write `this.assets = []` above line 18

    fn load_assets() -> Array[String]:
        return [f"atlas@{this.renderer.width}px #{this.assets.len()}"]

fn main():
    g = Game(800)
    println(g.assets[0])
