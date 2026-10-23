# A4: removing from a list while iterating skips elements
hp = [0, 0, 5]
for x in hp:
    if x <= 0:
        hp.remove(x)
print("A4 list after remove-in-loop:", hp)
# A4: growing a dict while iterating
d = {"a": 1}
try:
    for k in d:
        d[k + "x"] = 1
except RuntimeError as e:
    print("A4 dict:", e)
# A21: counts[w] += 1 on a missing key
c = {}
try:
    c["w"] += 1
except KeyError as e:
    print("A21 KeyError:", e)
# A11: assignment aliases
items = [1, 2]; snap = items; items.append(3); print("A11 alias:", snap)
