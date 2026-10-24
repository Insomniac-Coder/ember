# A4: what the answer compiles the loop into, written by hand: an in-place filter with two
# positions (read r, write w) over the live length. Today's rules accept it.
struct Enemy:
    hp: int
    splits: bool

fn update(mut enemies: Array[Enemy]):
    r = 0
    w = 0
    while r < enemies.len():
        enemies[r].hp -= 1
        keep = true
        if enemies[r].hp <= 0:
            if enemies[r].splits:
                enemies.push(Enemy(hp=5, splits=false))
            keep = false                     # enemies.remove(e): the current element leaves
        if keep:
            if w != r:
                enemies.swap(w, r)
            w += 1
        r += 1
    enemies.truncate(w)

fn main():
    es = [Enemy(hp=1, splits=true), Enemy(hp=3, splits=false)]
    update(es)
    println(es.len())
