#$ test: compile-fail
#$ rules: R5
#$ help: fn load_assets() reads renderer, frame -> Array[String]:
## R5, row 7: `reads renderer` says exactly what the method reads; a body that reads another field does
## not match its first line (AN-31, E2105).

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
        return [f"atlas@{this.renderer.width}px #{this.frame}"]    #$ error[E2105]: `load_assets` reads `this.frame`, which its first line does not name; write `fn load_assets() reads renderer, frame -> Array[String]:`

fn main():
    g = Game(800)
    println(g.assets[0])
