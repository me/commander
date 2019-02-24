use std::cmp::Ordering;

pub struct Matcher {
  count: usize,
}

#[derive(Clone, Debug)]
pub struct MatchResult {
  pub line: String,
  pub score: usize,
  pub matches: Vec<(usize, usize)>,
}

impl Ord for MatchResult {
  fn cmp(&self, other: &MatchResult) -> Ordering {
    self.score.cmp(&other.score)
  }
}

impl PartialOrd for MatchResult {
  fn partial_cmp(&self, other: &MatchResult) -> Option<Ordering> {
    self.score.partial_cmp(&other.score)
  }
}

impl PartialEq for MatchResult {
  fn eq(&self, other: &MatchResult) -> bool {
    self.score == other.score
  }
}

impl Eq for MatchResult {}

impl Matcher {
  pub fn new(count: usize) -> Matcher {
    Matcher { count }
  }

  pub fn run(&self, lines: &[String], query: &Option<String>) -> Vec<MatchResult> {
    let mut matches: Vec<MatchResult> = lines
      .iter()
      .map(|line| self.score(line, query))
      .filter_map(|v| v)
      .collect();
    matches.sort_unstable();
    matches.truncate(self.count);
    matches
  }

  pub fn score(&self, line: &str, query: &Option<String>) -> Option<MatchResult> {
    match query {
      Some(query) => match line.find(query) {
        Some(m) => Some(MatchResult {
          line: line.to_string(),
          score: m,
          matches: vec![(m, m + line.len())],
        }),
        None => None,
      },
      None => Some(MatchResult {
        line: line.to_string(),
        score: 0,
        matches: Vec::new(),
      }),
    }
  }
}
