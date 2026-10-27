#$ test: compile-fail
#$ rules: R5
#$ help: fn load_assets() reads renderer -> Array[String]:
## R5, row 7: a method with no `reads` may read any field, so `init` may not call it before every
## field is set; the message gives the `reads` line (AN-31, E2100).

class Renderer:
    width: int

class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(width: int):
        this.renderer = Renderer(width=width)
        this.assets = this.load_assets()    #$ error[E2100]: `load_assets` may read any field of `Game`, and `assets` is not set yet; say what it reads: `fn load_assets() reads renderer -> Array[String]:`

    fn load_assets() -> Array[String]:
        return [f"atlas@{this.renderer.width}px"]

fn main():
    g = Game(800)
    println(g.assets[0])
