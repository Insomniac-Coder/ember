# PROPOSED syntax. Each task changes its own class object (4.7, "claimed objects", discarded there).
@parallel
for p in players:            # players: Array[Player], Player a class; the list may hold one object twice
    p.hp -= 1
    p.x += p.hp & 1
