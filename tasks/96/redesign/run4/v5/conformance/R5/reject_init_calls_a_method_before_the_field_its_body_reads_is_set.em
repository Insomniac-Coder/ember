#$ test: compile-fail
#$ rules: R5
#$ help: give it a value first: move line 17 above line 16
## R5, row 7: `init` may call a method of the object it builds only once every field that method's body
## reads is set, worked out from the body (verdict 86; AN-31, E2100).

class Renderer:
    width: int

class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(width: int):
        this.assets = this.load_assets()    #$ error[E2100]: `init` calls `load_assets()` (line 16), which reads `this.renderer` (line 20), and `renderer` is set only on line 17
        this.renderer = Renderer(width=width)

    fn load_assets() -> Array[String]:
        return [f"atlas@{this.renderer.width}px"]

fn main():
    g = Game(800)
    println(g.assets[0])
