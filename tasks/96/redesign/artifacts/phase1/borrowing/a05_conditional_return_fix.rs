// A5 (ii) workaround: look up twice, or use the entry API.
use std::collections::HashMap;

pub fn get_or_load(cache: &mut HashMap<u32, String>, id: u32) -> &String {
    if cache.contains_key(&id) {
        return cache.get(&id).unwrap();
    }
    cache.insert(id, format!("asset {}", id));
    cache.get(&id).unwrap()
}
pub fn get_or_load_entry(cache: &mut HashMap<u32, String>, id: u32) -> &String {
    cache.entry(id).or_insert_with(|| format!("asset {}", id))
}
