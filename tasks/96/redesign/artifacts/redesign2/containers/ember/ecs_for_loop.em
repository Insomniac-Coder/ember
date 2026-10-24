# ECS: the same step written as a view loop that names the components (pos, vel), as EnTT's
# `for (auto [e, pos, vel] : view.each())` and as a Python programmer writes it.
struct V2:
    x: f32
    y: f32

struct Pool:
    sparse: Array[u32]
    ents: Array[u32]
    data: Array[V2]

    fn del(mut self, id: u32):
        i = self.sparse[id as int] as int
        last = self.ents.len() - 1
        moved = self.ents[last]
        self.ents[i] = moved
        self.data[i] = self.data[last]
        self.sparse[moved as int] = i as u32
        self.sparse[id as int] = 0xFFFF_FFFFu32
        _e = self.ents.pop()
        _d = self.data.pop()

struct Reg:
    pos: Pool
    vel: Pool

    fn step(mut self, s: u32):
        for j, id in enumerate(self.vel.ents):
            pos = ref mut self.pos.data[self.pos.sparse[id as int] as int]
            vel = ref self.vel.data[j]
            pos.x += vel.x
            pos.y += vel.y
            if id % 100u32 == s:
                self.pos.del(id)
                self.vel.del(id)

fn main():
    reg = Reg(pos=Pool(sparse=[], ents=[], data=[]), vel=Pool(sparse=[], ents=[], data=[]))
    reg.step(0u32)
    println(reg.pos.ents.len())
