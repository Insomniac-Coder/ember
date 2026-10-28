#$ test: run-pass
#$ rules: FN-4
#$ stdout: atlas@800px
## design rules: R5
## R5, row 7: `load_assets`'s body reads only `this.renderer` (worked out, verdict 86), so `init` may
## call it once `renderer` is set; `init` sets its own fields through `this` with no word (AN-31's NEW).

class Renderer:
    width: int

class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(width: int):
        this.renderer = Renderer(width=width)
        this.assets = this.load_assets()

    fn load_assets() -> Array[String]:
        return [f"atlas@{this.renderer.width}px"]

fn main():
    g = Game(800)
    println(g.assets[0])
