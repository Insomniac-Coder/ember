# The ownership group's answers in one program, written as a Python programmer writes it.
# `ember check` shows what today's Ember refuses; the report says what each line needs.
from std.collections import Map

class Player:                                   # shared: the camera and the HUD both hold it (A9, A10)
    name: String
    hp: int = 100
    inventory: Array[String] = []

class Camera:
    target: Player

struct Mesh:
    verts: Array[f32]

    fn radius(self) -> f32:                     # A31: a cache filled by a read-only method
        if self.radius_cache.is_none():
            r: f32 = 0.0
            for v in self.verts:
                r = max(r, abs(v))
            self.radius_cache = Some(r)
        return self.radius_cache.unwrap()

    radius_cache: Option[f32] = None

struct Node:
    value: int
    children: Array[Node] = []

struct Game:
    players: Array[Player]
    log: Array[String]
    kills: Map[String, int]
    frame: int = 0

    fn init(mut self, names: Array[String]):    # A18: a helper called before every field is set
        self.players = []
        for n in names:
            self.players.push(Player(name=n.clone()))
        self.log = self.first_lines()
        self.kills = Map[String, int]()

    fn first_lines(self) -> Array[String]:
        return [f"{self.players.len()} players"]

    fn note(mut self, msg: String):             # A1: touches only `log` and `frame`
        self.log.push(f"[{self.frame}] {msg}")

    fn tick(mut self):
        for p in self.players:
            p.hp -= 1
            if p.hp <= 0:
                self.note(f"{p.name} died")     # A1
                self.kills[p.name] = self.kills.get(p.name, 0) + 1   # A21

fn build_tree() -> Node:
    root = Node(value=0)
    child = Node(value=1)
    root.children.push(child)
    child.value = 5                             # A11: Python changes root.children[0]
    return root

fn main():
    g = Game(["ann", "bob"])
    cam = Camera(g.players[0])                  # A9, A34: a second handle to one player
    cam.target.hp -= 100
    g.tick()
    snapshot = g.log                            # A11: a second name for one list
    g.log.push("end")
    println(snapshot)
    m = Mesh(verts=[1.0, -2.0])
    println(m.radius())
    println(build_tree().children[0].value)
