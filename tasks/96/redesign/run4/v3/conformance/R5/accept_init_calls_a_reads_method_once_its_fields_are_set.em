#$ test: run-pass
#$ rules: R5
#$ stdout: atlas@800px
## R5, row 7: `reads renderer` says exactly what the helper reads; `init` may call it once those
## fields are set, and sets its own fields through `this` with no word (AN-31's NEW).

class Renderer:
    width: int

class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(width: int):
        this.renderer = Renderer(width=width)
        this.assets = this.load_assets()

    fn load_assets() reads renderer -> Array[String]:
        return [f"atlas@{this.renderer.width}px"]

fn main():
    g = Game(800)
    println(g.assets[0])
