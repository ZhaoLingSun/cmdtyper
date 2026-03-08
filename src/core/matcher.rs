use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchResult {
    Exact(usize),
    Normalized(usize),
    NoMatch {
        closest: String,
        diff: Vec<DiffSegment>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffSegment {
    pub text: String,
    pub kind: DiffKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    Same,
    Added,
    Removed,
}

pub struct Matcher;

pub fn normalize(input: &str) -> String {
    Matcher::normalize(input)
}

pub fn check(input: &str, answers: &[String]) -> MatchResult {
    Matcher::check(input, answers)
}

impl Matcher {
    pub fn normalize(input: &str) -> String {
        input
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }

    pub fn check(input: &str, answers: &[String]) -> MatchResult {
        if let Some(index) = answers.iter().position(|answer| answer == input) {
            return MatchResult::Exact(index);
        }

        let normalized_input = Self::normalize(input);

        if let Some(index) = answers
            .iter()
            .position(|answer| Self::normalize(answer) == normalized_input)
        {
            return MatchResult::Normalized(index);
        }

        let fallback = answers
            .iter()
            .enumerate()
            .map(|(index, answer)| {
                let normalized_answer = Self::normalize(answer);
                let score = lcs_len(&normalized_input, &normalized_answer);
                let len_delta = normalized_input
                    .chars()
                    .count()
                    .abs_diff(normalized_answer.chars().count());
                ClosestCandidate {
                    index,
                    score,
                    len_delta,
                    original: answer,
                    normalized: normalized_answer,
                }
            })
            .max_by(ClosestCandidate::cmp)
            .map(|candidate| {
                (
                    candidate.original.to_string(),
                    diff_strings(&normalized_input, &candidate.normalized),
                )
            });

        match fallback {
            Some((closest, diff)) => MatchResult::NoMatch { closest, diff },
            None => MatchResult::NoMatch {
                closest: String::new(),
                diff: diff_strings(&normalized_input, ""),
            },
        }
    }
}

struct ClosestCandidate<'a> {
    index: usize,
    score: usize,
    len_delta: usize,
    original: &'a str,
    normalized: String,
}

impl ClosestCandidate<'_> {
    fn cmp(left: &Self, right: &Self) -> Ordering {
        left.score
            .cmp(&right.score)
            .then_with(|| right.len_delta.cmp(&left.len_delta))
            .then_with(|| right.index.cmp(&left.index))
    }
}

fn lcs_len(left: &str, right: &str) -> usize {
    let left_chars: Vec<char> = left.chars().collect();
    let right_chars: Vec<char> = right.chars().collect();
    let table = lcs_table(&left_chars, &right_chars);
    table[left_chars.len()][right_chars.len()]
}

fn diff_strings(input: &str, answer: &str) -> Vec<DiffSegment> {
    let input_chars: Vec<char> = input.chars().collect();
    let answer_chars: Vec<char> = answer.chars().collect();
    let table = lcs_table(&input_chars, &answer_chars);

    let mut operations = Vec::new();
    let (mut i, mut j) = (input_chars.len(), answer_chars.len());

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && input_chars[i - 1] == answer_chars[j - 1] {
            operations.push(DiffOp {
                ch: input_chars[i - 1],
                kind: DiffKind::Same,
            });
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || table[i][j - 1] >= table[i - 1][j]) {
            operations.push(DiffOp {
                ch: answer_chars[j - 1],
                kind: DiffKind::Added,
            });
            j -= 1;
        } else if i > 0 {
            operations.push(DiffOp {
                ch: input_chars[i - 1],
                kind: DiffKind::Removed,
            });
            i -= 1;
        }
    }

    operations.reverse();
    group_ops(operations)
}

#[derive(Debug, Clone, Copy)]
struct DiffOp {
    ch: char,
    kind: DiffKind,
}

fn group_ops(ops: Vec<DiffOp>) -> Vec<DiffSegment> {
    let mut grouped: Vec<DiffSegment> = Vec::new();

    for op in ops {
        match grouped.last_mut() {
            Some(last) if last.kind == op.kind => last.text.push(op.ch),
            _ => grouped.push(DiffSegment {
                text: op.ch.to_string(),
                kind: op.kind,
            }),
        }
    }

    grouped
}

fn lcs_table(left: &[char], right: &[char]) -> Vec<Vec<usize>> {
    let mut table = vec![vec![0; right.len() + 1]; left.len() + 1];

    for (i, left_char) in left.iter().enumerate() {
        for (j, right_char) in right.iter().enumerate() {
            table[i + 1][j + 1] = if left_char == right_char {
                table[i][j] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }

    table
}

#[cfg(test)]
mod tests {
    use super::{check, normalize, DiffKind, DiffSegment, MatchResult, Matcher};

    fn reconstruct_input(diff: &[DiffSegment]) -> String {
        diff.iter()
            .filter(|segment| matches!(segment.kind, DiffKind::Same | DiffKind::Removed))
            .map(|segment| segment.text.as_str())
            .collect::<String>()
    }

    fn reconstruct_answer(diff: &[DiffSegment]) -> String {
        diff.iter()
            .filter(|segment| matches!(segment.kind, DiffKind::Same | DiffKind::Added))
            .map(|segment| segment.text.as_str())
            .collect::<String>()
    }

    #[test]
    fn normalize_trims_collapses_spaces_and_lowercases() {
        assert_eq!(normalize("  LS   -LA   /Var/Log  "), "ls -la /var/log");
        assert_eq!(Matcher::normalize("A\tB\nC"), "a b c");
    }

    #[test]
    fn check_prefers_exact_match() {
        let answers = vec!["ls -la /tmp".to_string(), "pwd".to_string()];

        assert_eq!(check("ls -la /tmp", &answers), MatchResult::Exact(0));
    }

    #[test]
    fn check_falls_back_to_normalized_match() {
        let answers = vec!["ls -la /var/log".to_string(), "pwd".to_string()];

        assert_eq!(
            check("  LS   -LA   /VAR/LOG  ", &answers),
            MatchResult::Normalized(0)
        );
    }

    #[test]
    fn no_match_returns_closest_answer_and_added_segments() {
        let answers = vec!["ls -la /tmp".to_string(), "pwd".to_string()];

        match check("ls /tmp", &answers) {
            MatchResult::NoMatch { closest, diff } => {
                assert_eq!(closest, "ls -la /tmp");
                assert_eq!(reconstruct_input(&diff), "ls /tmp");
                assert_eq!(reconstruct_answer(&diff), "ls -la /tmp");
                assert!(
                    diff.iter()
                        .any(|segment| segment.kind == DiffKind::Added
                            && segment.text.contains("-la"))
                );
            }
            other => panic!("expected NoMatch, got {other:?}"),
        }
    }

    #[test]
    fn no_match_returns_removed_segments_for_extra_input() {
        let answers = vec!["ls -la /tmp".to_string()];

        match check("ls -laa /tmp", &answers) {
            MatchResult::NoMatch { closest, diff } => {
                assert_eq!(closest, "ls -la /tmp");
                assert_eq!(reconstruct_input(&diff), "ls -laa /tmp");
                assert_eq!(reconstruct_answer(&diff), "ls -la /tmp");
                assert!(diff
                    .iter()
                    .any(|segment| { segment.kind == DiffKind::Removed && segment.text == "a" }));
                assert!(
                    diff.iter()
                        .filter(|segment| segment.kind == DiffKind::Removed)
                        .count()
                        >= 1
                );
            }
            other => panic!("expected NoMatch, got {other:?}"),
        }
    }

    #[test]
    fn no_match_chooses_answer_with_best_lcs_score() {
        let answers = vec!["git stash".to_string(), "git status".to_string()];

        match check("git sttus", &answers) {
            MatchResult::NoMatch { closest, .. } => assert_eq!(closest, "git status"),
            other => panic!("expected NoMatch, got {other:?}"),
        }
    }

    #[test]
    fn empty_answers_return_removed_input_diff() {
        match check("echo hello", &[]) {
            MatchResult::NoMatch { closest, diff } => {
                assert!(closest.is_empty());
                assert_eq!(
                    diff,
                    vec![DiffSegment {
                        text: "echo hello".to_string(),
                        kind: DiffKind::Removed,
                    }]
                );
            }
            other => panic!("expected NoMatch, got {other:?}"),
        }
    }
}
