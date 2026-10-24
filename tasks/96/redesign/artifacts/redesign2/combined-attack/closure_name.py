# closure_name.py: the Python twin of closure_name.c (problem 4 of phase3b-combined-attack.md).
class Enemy:
    def __init__(self, hp): self.hp = hp

class Game:
    def __init__(self): self.enemies = []; self.buttons = []

def setup(g):
    boss = g.enemies[0]
    def on_hit():
        boss.hp -= 1
    g.buttons.append(on_hit)

def wave(g):
    g.enemies.pop(0)
    for i in range(64): g.enemies.append(Enemy(i))

g = Game(); g.enemies.append(Enemy(10)); g.enemies.append(Enemy(20))
setup(g)
boss = g.enemies[0]
wave(g)
for b in g.buttons: b()
print("python: enemies[0].hp =", g.enemies[0].hp, "; the boss (out of the list) hp =", boss.hp)
