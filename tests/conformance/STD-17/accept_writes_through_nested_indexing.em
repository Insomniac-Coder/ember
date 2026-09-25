#$ test: run-pass
#$ rules: STD-17, STD-16, TYP-20
#$ stdout: {'x': {'y': 2, 'z': 3}} {'a': [5, 6]} [0, 7, 0]
# `[STD-17]` — a written `a[i]` goes through `index_mut`, and so does every
# `m[k]` it is reached through: `m[a][b] = v`, `m[k].push(x)`, `m[k][0] = v`,
# `ref mut m[k]`. An index used only to read (`xs[m[k]] = v`) stays `index`.

fn place(m: Map[String, int]) -> Array[int]:
    xs = [0, 0, 0]
    xs[m["b"]] = 7
    return xs

fn main():
    m: Map[String, Map[String, int]] = {"x": {"y": 1}}
    m["x"]["y"] = 2
    m["x"]["z"] = 3
    lists: Map[String, Array[int]] = {"a": [0]}
    lists["a"][0] = 5
    r = ref mut lists["a"]
    r.push(6)
    println(m, lists, place({"a": 0, "b": 1}))
