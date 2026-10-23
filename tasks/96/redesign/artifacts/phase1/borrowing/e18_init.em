# A18: call a method during construction, before every field is set.
struct Renderer:
    width: int

struct Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(mut self, width: int):
        self.renderer = Renderer(width=width)
        self.assets = self.load_assets()

    fn load_assets(self) -> Array[String]:
        return [f"atlas@{self.renderer.width}px"]

fn main():
    g = Game(640)
    println(g.assets)
