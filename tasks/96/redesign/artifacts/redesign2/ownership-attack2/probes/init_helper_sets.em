# Attack on A18: Python's commonest constructor helper sets fields, it does not only read them.
struct Game:
    width: int
    assets: Array[String]
    sounds: Array[String]

    fn init(mut self, width: int):
        self.width = width
        self.load()                       # sets assets and sounds

    fn load(mut self):
        self.assets = [f"atlas@{self.width}px"]
        self.sounds = ["boom.wav"]

fn main():
    g = Game(640)
    println(g.assets, g.sounds)
