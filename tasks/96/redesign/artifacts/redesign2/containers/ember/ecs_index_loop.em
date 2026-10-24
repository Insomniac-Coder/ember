# ECS (EnTT layout): the view loop as the library would write it under the answer: walk the
# Velocity pool backwards by position, look Position up through `sparse`, and destroy and create
# entities inside the loop. No reference is held across a change, so today's rules accept it.
struct V2:
    x: f32
    y: f32

struct Pool:
    sparse: Array[u32]
    ents: Array[u32]
    data: Array[V2]

    fn add(mut self, id: u32, v: V2):
        while self.sparse.len() <= id as int:
            self.sparse.push(0xFFFF_FFFFu32)
        self.sparse[id as int] = self.ents.len() as u32
        self.ents.push(id)
        self.data.push(v)

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
    next: u32

    fn step(mut self, s: u32):
        j = self.vel.ents.len()
        while j > 0:
            j -= 1
            id = self.vel.ents[j]
            if id as int >= self.pos.sparse.len():
                continue
            pi = self.pos.sparse[id as int]
            if pi == 0xFFFF_FFFFu32:
                continue
            self.pos.data[pi as int].x += self.vel.data[j].x
            self.pos.data[pi as int].y += self.vel.data[j].y
            if id % 100u32 == s:
                self.pos.del(id)                 # despawn the current entity
                self.vel.del(id)
                e = self.next                    # spawn a new one (appended: not visited)
                self.next += 1u32
                self.pos.add(e, V2(x=0.0f32, y=0.0f32))
                self.vel.add(e, V2(x=1.0f32, y=1.0f32))

fn main():
    reg = Reg(pos=Pool(sparse=[], ents=[], data=[]), vel=Pool(sparse=[], ents=[], data=[]), next=10u32)
    for i in 0u32..10u32:
        reg.pos.add(i, V2(x=i as f32, y=0.0f32))
        if i % 2u32 == 0u32:
            reg.vel.add(i, V2(x=1.0f32, y=0.5f32))
    for s in 0u32..100u32:
        reg.step(s)
    println(reg.pos.ents.len())
