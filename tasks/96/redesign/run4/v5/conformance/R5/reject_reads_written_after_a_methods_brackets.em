#$ test: compile-fail
#$ rules: R5
#$ help: write `fn load_assets() -> Array[String]:`
## R5, verdict 86: `reads` is not a word of Ember: the compiler works out what a method reads from its
## body (row 7: `fn load_assets() -> Array[String]:`). A first line that still writes it is refused with
## the line to write (E0100). The nearest program that is allowed is
## `accept_init_calls_a_method_once_the_fields_its_body_reads_are_set`.

class Renderer:
    width: int

class Game:
    renderer: Renderer
    assets: Array[String]

    fn init(width: int):
        this.renderer = Renderer(width=width)
        this.assets = this.load_assets()

    fn load_assets() reads renderer -> Array[String]:    #$ error[E0100]: `reads renderer` is not Ember: the compiler works out what `load_assets` reads from its body
        return [f"atlas@{this.renderer.width}px"]

fn main():
    g = Game(800)
    println(g.assets[0])
