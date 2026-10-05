#$ test: run-pass
#$ rules: STD-19
#$ stdout: Some(3) Some(2) 2
#$ stdout: None None
#$ stdout: Some(0) None
#$ stdout: Some(30) Some(20)
#$ stdout: Some(1) None
#$ profiles: debug, release
# G8-4 (the owner's design, 0.9.10) — `nth_back(n)` leaves out `n` items from the back and gives
# the one before them, in the iterator's count type: a range moves its end past them at once (the
# whole of `0 as u64 ..= u64.MAX` but its first value in one step), any other iterator steps back
# one at a time. Past the front it is `None`, and the iterator is empty.

fn main():
    five = (0 .. 5).iter()
    println(five.nth_back(1 as u64), five.next_back(), five.len())
    gone = (0 .. 5).iter()
    println(gone.nth_back(5 as u64), gone.next())
    whole = (0 as u64 ..= u64.MAX).iter()
    println(whole.nth_back(u64.MAX as u128), whole.next())
    xs: Array[int] = [10, 20, 30, 40]
    stored = xs.iter().copied()
    println(stored.nth_back(1), stored.nth_back(0))
    small = (1 as u8 ..= 3 as u8).iter()
    println(small.nth_back(2), small.next())
