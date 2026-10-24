pub fn first_len(line: &str) -> usize { line.split(' ').next().unwrap_or("").len() }
