// A21: updating a map value in place by key.
use std::collections::HashMap;

pub fn count_words(text: &str) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for w in text.split_whitespace() {
        counts[w] += 1;
    }
    counts
}
