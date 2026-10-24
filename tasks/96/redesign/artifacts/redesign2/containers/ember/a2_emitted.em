# A2: what the answer compiles `boss` into, written by hand: the place enemies[0], looked up again
# after the push. This is the program the compiler would produce; today's rules accept it.
struct Enemy:
    x: float
    hp: int

fn split_boss(mut enemies: Array[Enemy]):
    b = 0                                  # boss's position, computed once
    enemies.push(Enemy(x=enemies[b].x + 1.0, hp=enemies[b].hp // 2))
    enemies[b].hp = enemies[b].hp // 2      # enemies[b] in the buffer as it is after the push

fn main():
    es = [Enemy(x=0.0, hp=100)]
    split_boss(es)
    println(es.len())
