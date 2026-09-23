use std::collections::HashMap;

/// `[DIA-12]` N1 — rank in-scope candidates by Damerau–Levenshtein distance,
/// then declaration proximity. Only close spellings are offered, at most
/// three, with a lexical tie-break so output is deterministic.
pub(super) fn rank(
    written: &str,
    candidates: impl IntoIterator<Item = (String, usize)>,
) -> Vec<String> {
    let written_length = written.chars().count();
    let mut unique = HashMap::new();
    for (candidate, proximity) in candidates {
        if candidate == written {
            continue;
        }
        unique
            .entry(candidate)
            .and_modify(|existing: &mut usize| *existing = (*existing).min(proximity))
            .or_insert(proximity);
    }

    let mut ranked: Vec<_> = unique
        .into_iter()
        .filter_map(|(candidate, proximity)| {
            let distance = damerau_levenshtein(written, &candidate);
            (distance <= 2 || distance * 3 <= written_length)
                .then_some((distance, proximity, candidate))
        })
        .collect();
    ranked
        .sort_by_key(|(distance, proximity, candidate)| (*distance, *proximity, candidate.clone()));
    ranked
        .into_iter()
        .take(3)
        .map(|(_, _, candidate)| candidate)
        .collect()
}

/// Unrestricted Damerau–Levenshtein distance over Unicode scalar values.
fn damerau_levenshtein(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    let (rows, columns) = (left.len(), right.len());
    let sentinel = rows + columns;
    let mut distance = vec![vec![0; columns + 2]; rows + 2];

    distance[0][0] = sentinel;
    for row in 0..=rows {
        distance[row + 1][0] = sentinel;
        distance[row + 1][1] = row;
    }
    for column in 0..=columns {
        distance[0][column + 1] = sentinel;
        distance[1][column + 1] = column;
    }

    let mut last_row: HashMap<char, usize> = HashMap::new();
    for row in 1..=rows {
        let mut last_match_column = 0;
        for column in 1..=columns {
            let previous_row = *last_row.get(&right[column - 1]).unwrap_or(&0);
            let previous_column = last_match_column;
            let substitution_cost = if left[row - 1] == right[column - 1] {
                last_match_column = column;
                0
            } else {
                1
            };

            distance[row + 1][column + 1] = (distance[row][column] + substitution_cost)
                .min(distance[row + 1][column] + 1)
                .min(distance[row][column + 1] + 1)
                .min(
                    distance[previous_row][previous_column]
                        + (row - previous_row - 1)
                        + 1
                        + (column - previous_column - 1),
                );
        }
        last_row.insert(left[row - 1], row);
    }

    distance[rows + 1][columns + 1]
}

#[cfg(test)]
mod tests {
    use super::{damerau_levenshtein, rank};

    #[test]
    fn damerau_distance_counts_an_adjacent_transposition_once() {
        assert_eq!(damerau_levenshtein("coutn", "count"), 1);
    }

    #[test]
    fn damerau_distance_supports_repeated_transpositions() {
        assert_eq!(damerau_levenshtein("CA", "ABC"), 2);
    }

    #[test]
    fn suggestions_rank_by_distance_then_proximity_and_cap_at_three() {
        let suggestions = rank(
            "pritn",
            [
                ("print".to_owned(), 20),
                ("prtin".to_owned(), 10),
                ("prain".to_owned(), 1),
                ("wrong".to_owned(), 0),
            ],
        );

        assert_eq!(suggestions, ["prtin", "print", "prain"]);
    }
}
