# A4, the Python habit: walk a copy of the list while changing the list (for e in enemies[:]).
# Ember's enemies[..] is a view, not a copy (spec line 1928).
struct Enemy:
    hp: int

fn update(mut enemies: Array[Enemy]):
    for e in enemies[..]:
        e.hp -= 1
        if e.hp <= 0:
            _gone = enemies.remove(0)   # the answer's spelling: enemies.remove(e)
            enemies.push(Enemy(hp=9))

fn main():
    es = [Enemy(hp=1), Enemy(hp=3)]
    update(es)
    println(es.len())
