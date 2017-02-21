use std::path::Path;
use url::Url;

pub mod git;


#[derive(Debug)]
pub enum Vcs {
  Git,
  Subversion,
  Mercurial,
  Darcs,
}

impl ::std::str::FromStr for Vcs {
  type Err = String;
  fn from_str(s: &str) -> Result<Vcs, String> {
    match s {
      "git" => Ok(Vcs::Git),
      "svn" => Ok(Vcs::Subversion),
      "hg" => Ok(Vcs::Mercurial),
      "darcs" => Ok(Vcs::Darcs),
      s => Err(format!("{} is invalid string", s)),
    }
  }
}

trait StrSkip {
  fn skip<'a>(&'a self, n: usize) -> &'a str;
}

impl StrSkip for str {
  fn skip<'a>(&'a self, n: usize) -> &'a str {
    let mut s = self.chars();
    for _ in 0..n {
      s.next();
    }
    s.as_str()
  }
}

#[test]
fn test_skipped_1() {
  assert_eq!("hoge".skip(1), "oge");
  assert_eq!("あいueo".skip(1), "いueo");
}

pub fn detect_from_path(path: &Path) -> Option<Vcs> {
  [".git", ".svn", ".hg", "_darcs"]
    .into_iter()
    .find(|vcs| path.join(vcs).exists())
    .and_then(|s| s.skip(1).parse().ok())
}

pub fn detect_from_remote(_: &Url) -> Option<Vcs> {
  None
}