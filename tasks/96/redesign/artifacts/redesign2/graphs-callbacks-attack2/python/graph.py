# graph.py: bench/graph.c's workload in Python, with the reverse lists kept by hand, to check that the
# C programs do this work: it must print the same checksum. Run: python -I graph.py
N, R, PASSES = 100_000, 8, 10
state = 11
def rand():
    global state
    state = (state * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
    return state >> 33

class City:
    def __init__(self, id):
        self.id = id; self.roads = []; self.into = []

def road_push(a, b):
    a.roads.append(b); b.into.append(a)

def city_die(x):
    for y in x.into:
        if y is not x: y.roads.remove(x)
    for z in x.roads:
        if z is not x: z.into.remove(x)

cities = [City(i) for i in range(N)]
for i in range(N):
    for _ in range(R):
        road_push(cities[i], cities[rand() % N])
next_id, total = N, 0
for p in range(PASSES):
    for c in cities:
        for r in c.roads: total += r.id
    for _ in range(N // 100):
        k = rand() % N
        old = cities[k]
        cities[k] = City(next_id); next_id += 1
        city_die(old)
        for _ in range(R):
            road_push(cities[k], cities[rand() % N])
for c in cities:
    total += len(c.roads) * 3 + len(c.into)
print(total & 0xFFFFFFFFFFFFFFFF)
