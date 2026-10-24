// A15: scoped threads push to one Vec (Python: results.append).
pub fn squares() -> Vec<i32> {
    let mut results = Vec::new();
    std::thread::scope(|s| {
        for i in 0..4 {
            s.spawn(|| results.push(i * i));
        }
    });
    results
}
