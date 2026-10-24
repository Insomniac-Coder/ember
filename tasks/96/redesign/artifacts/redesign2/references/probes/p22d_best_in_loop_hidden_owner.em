# A22: the rewrite of p22c with the owner following the view (a hidden owner beside `best`).
fn longest(lines: Array[String]) -> int:
    best_owner: String = ""
    for line in lines:
        tmp = line.to_lower()
        if tmp.trim().len() > best_owner.trim().len():
            best_owner = tmp
    best = best_owner.trim()
    return best.len()

fn main():
    println(longest(["a ", " bb"]))
