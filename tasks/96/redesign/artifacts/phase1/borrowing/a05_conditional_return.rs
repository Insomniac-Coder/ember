// A5 (ii): NLL problem case #3, a conditionally returned borrow (asset cache).
use std::collections::HashMap;

pub fn get_or_load(cache: &mut HashMap<u32, String>, id: u32) -> &String {
    if let Some(s) = cache.get(&id) {
        return s;
    }
    cache.insert(id, format!("asset {}", id));
    cache.get(&id).unwrap()
}
