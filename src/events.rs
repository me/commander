use super::matcher::MatchResult;

pub enum FinderCommand {
  Refresh(Vec<String>),
  Query(String),
}

pub enum InterfaceCommand {
  Query(String),
  Results(Vec<MatchResult>),
  Stop,
}
