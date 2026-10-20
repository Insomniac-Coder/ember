#$ test: run-pass
#$ rules: STD-12, TYP-17
#$ profiles: debug, development
#$ stdout: true 1
#$ stdout: 2 true false
## `[STD-12]` (D-765) — a `Map[String, V]` looked up with a `str` uses `str`'s `AsKey[String]`, and a
## `Map[str, V]` its own key's: each instance of the map's methods calls through its own bounds,
## whichever is made first.

static NAMES: Array[str] = ["ann", "bob"]

fn main():
    owned_keys: Map[String, int] = {}
    owned_keys.insert(String.from("x"), 1)
    println(owned_keys.contains_key("x"), owned_keys.len())
    index: Map[str, int] = {name: i for i, name in NAMES.iter().enumerate()}
    println(index.len(), index.contains_key("ann"), index.contains_key("cat"))
