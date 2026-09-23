use std::cmp::Ordering;
use std::collections::HashMap;

use ember_span::Span;

/// A visible spelling paired with its declaration's source location and
/// canonical identity. The location determines locality; the identity orders
/// cross-file matches without comparing file-relative offsets.
pub(super) struct Candidate {
    pub(super) name: String,
    pub(super) declaration: Span,
    pub(super) canonical_name: String,
}

/// `[DIA-12]` N1 — rank in-scope candidates by Damerau–Levenshtein distance,
/// then same-file proximity or cross-file canonical identity. Only close
/// spellings are offered, at most three, with deterministic tie-breaks.
pub(super) fn rank(
    written: &str,
    use_span: Span,
    candidates: impl IntoIterator<Item = Candidate>,
) -> Vec<String> {
    let written_length = written.chars().count();
    let mut unique = HashMap::new();
    for candidate in candidates {
        if candidate.name == written {
            continue;
        }
        match unique.entry(candidate.name.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if compare_tie_breaks(use_span, &candidate, entry.get()) == Ordering::Less {
                    entry.insert(candidate);
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(candidate);
            }
        }
    }

    let mut ranked: Vec<_> = unique
        .into_values()
        .filter_map(|candidate| {
            let distance = damerau_levenshtein(written, &candidate.name);
            (distance <= 2 || distance * 3 <= written_length).then_some((distance, candidate))
        })
        .collect();
    ranked.sort_by(|(left_distance, left), (right_distance, right)| {
        left_distance
            .cmp(right_distance)
            .then_with(|| compare_tie_breaks(use_span, left, right))
    });
    ranked
        .into_iter()
        .take(3)
        .map(|(_, candidate)| candidate.name)
        .collect()
}

fn compare_tie_breaks(use_span: Span, left: &Candidate, right: &Candidate) -> Ordering {
    let left_same_file = left.declaration.file == use_span.file;
    let right_same_file = right.declaration.file == use_span.file;
    left_same_file
        .cmp(&right_same_file)
        .reverse()
        .then_with(|| {
            if left_same_file {
                left.declaration
                    .start
                    .abs_diff(use_span.start)
                    .cmp(&right.declaration.start.abs_diff(use_span.start))
            } else {
                left.canonical_name
                    .as_bytes()
                    .cmp(right.canonical_name.as_bytes())
            }
        })
        .then_with(|| left.name.as_bytes().cmp(right.name.as_bytes()))
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
    use ember_span::{FileId, Span};

    use super::{Candidate, damerau_levenshtein, rank};

    fn candidate(name: &str, file: u32, start: u32, canonical_name: &str) -> Candidate {
        Candidate {
            name: name.to_owned(),
            declaration: Span::new(FileId(file), start, start + name.len() as u32),
            canonical_name: canonical_name.to_owned(),
        }
    }

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
            Span::new(FileId(0), 100, 105),
            [
                candidate("print", 0, 120, "app::print"),
                candidate("prtin", 0, 90, "app::prtin"),
                candidate("prain", 0, 101, "app::prain"),
                candidate("wrong", 0, 100, "app::wrong"),
            ],
        );

        assert_eq!(suggestions, ["prtin", "print", "prain"]);
    }

    #[test]
    fn cross_file_ties_use_canonical_names_not_unrelated_offsets() {
        let suggestions = rank(
            "pritn",
            Span::new(FileId(0), 100, 105),
            [
                candidate("print", 1, 101, "text::print"),
                candidate("prtin", 2, 10_000, "io::prtin"),
            ],
        );

        assert_eq!(suggestions, ["prtin", "print"]);
    }

    #[test]
    fn equal_distance_same_file_candidates_precede_cross_file_candidates() {
        let suggestions = rank(
            "pritn",
            Span::new(FileId(0), 100, 105),
            [
                candidate("prtin", 1, 101, "io::prtin"),
                candidate("print", 0, 10_000, "app::print"),
            ],
        );

        assert_eq!(suggestions, ["print", "prtin"]);
    }

    #[test]
    fn equal_distance_same_file_candidates_keep_proximity_then_spelling_order() {
        let suggestions = rank(
            "pritn",
            Span::new(FileId(0), 100, 105),
            [
                candidate("prtin", 0, 100, "app::prtin"),
                candidate("print", 0, 100, "app::print"),
            ],
        );

        assert_eq!(suggestions, ["print", "prtin"]);
    }

    #[test]
    fn better_cross_file_spelling_precedes_worse_same_file_spelling() {
        let suggestions = rank(
            "pritn",
            Span::new(FileId(0), 100, 105),
            [
                candidate("print", 1, 1, "io::print"),
                candidate("printz", 0, 101, "app::printz"),
            ],
        );

        assert_eq!(suggestions, ["print", "printz"]);
    }

    #[test]
    fn cross_file_order_is_independent_of_candidate_and_file_discovery_order() {
        let use_span = Span::new(FileId(0), 500, 505);
        let first = rank(
            "pritn",
            use_span,
            [
                candidate("print", 1, 1, "support.io.print"),
                candidate("prtin", 2, 10_000, "support.text.print"),
            ],
        );
        let reversed = rank(
            "pritn",
            use_span,
            [
                candidate("prtin", 9, 1, "support.text.print"),
                candidate("print", 8, 10_000, "support.io.print"),
            ],
        );

        assert_eq!(first, ["print", "prtin"]);
        assert_eq!(reversed, first);
    }
}
