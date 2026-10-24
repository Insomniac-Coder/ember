# A8: the graph in today's Arena (refs between its own allocations).
struct City:
    name: int
    roads: Array[ref City]

fn main():
    g = Arena.with_capacity(4096)
    a = g.alloc(City(name=1, roads=[]))
    b = g.alloc(City(name=2, roads=[]))
    a.roads.push(b)
    b.roads.push(a)
    println(a.roads.len())
