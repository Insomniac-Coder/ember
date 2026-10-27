#$ test: compile-fail
#$ rules: R5
#$ help: fn load_assets() reads renderer -> Array[String]:
## R5: `reads` is checked against the body both ways: a field named that the body never reads is
## refused, so the line `init` relies on stays exact (E2105).

class Renderer:
    width: int

class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(width: int):
        this.renderer = Renderer(width=width)
        this.assets = this.load_assets()

    fn load_assets() reads renderer, frame -> Array[String]:    #$ error[E2105]: `load_assets` says `reads renderer, frame`, but it never reads `frame`; write `fn load_assets() reads renderer -> Array[String]:`
        return [f"atlas@{this.renderer.width}px"]

fn main():
    g = Game(800)
    println(g.assets[0])
