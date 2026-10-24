# A21 residue: Python's dict.get with a list default (groups.get("z", [])).
fn main():
    groups: Map[String, Array[String]] = Map()
    groups["a"] = ["ab"]
    println(groups.get_or("z", []))
