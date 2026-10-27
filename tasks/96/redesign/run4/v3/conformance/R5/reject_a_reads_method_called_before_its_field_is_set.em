#$ test: compile-fail
#$ rules: R5
#$ help: this.renderer = Renderer(width=width)
## R5, row 7: `init` may call a `reads` method only once every field it names is set (AN-31, E2100).

class Renderer:
    width: int

class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(width: int):
        this.assets = this.load_assets()    #$ error[E2100]: `load_assets` reads `renderer`, which `init` has not set yet; write `this.renderer = Renderer(width=width)` before this line
        this.renderer = Renderer(width=width)

    fn load_assets() reads renderer -> Array[String]:
        return [f"atlas@{this.renderer.width}px"]

fn main():
    g = Game(800)
    println(g.assets[0])
