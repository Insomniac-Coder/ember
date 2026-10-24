# A18: a helper that reads only fields already set, called during init; three spellings.
struct Renderer:
    width: int

struct GameA:
    renderer: Renderer
    assets: Array[String]

    fn init(mut self, width: int):
        self.renderer = Renderer(width=width)
        self.assets = self.load_assets()        # the Python spelling: a method on self

    fn load_assets(self) -> Array[String]:
        return [f"atlas@{self.renderer.width}px"]

fn load_assets_for(r: Renderer) -> Array[String]:
    return [f"atlas@{r.width}px"]

struct GameB:
    renderer: Renderer
    assets: Array[String]

    fn init(mut self, width: int):
        self.renderer = Renderer(width=width)
        self.assets = load_assets_for(self.renderer)   # Rust's workaround: pass the part

struct GameC:
    renderer: Renderer
    assets: Array[String] = []

    fn init(mut self, width: int):
        self.renderer = Renderer(width=width)
        self.assets = self.load_assets()        # every other field has a default

    fn load_assets(self) -> Array[String]:
        return [f"atlas@{self.renderer.width}px"]

fn main():
    a = GameA(640)
    b = GameB(640)
    c = GameC(640)
    println(a.assets, b.assets, c.assets)
