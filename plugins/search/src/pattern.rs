//! Additional Supported Search Patterns

use rmenu_plugin::Entry;

use crate::bang::Bang;

trait Pattern {
    fn is_match(&self, search: &str, bang: Option<&Bang>) -> Option<Vec<Entry>>;
}

pub struct Patterns {
    patterns: Vec<Box<dyn Pattern>>,
}

impl Patterns {
    pub fn new() -> Self {
        let patterns: Vec<Box<dyn Pattern>> = vec![Box::new(GithubRepoPattern::new())];
        Self { patterns }
    }
    pub fn try_match(&self, search: &str, bang: Option<&Bang>) -> Option<Vec<Entry>> {
        for pattern in self.patterns.iter() {
            if let Some(entries) = pattern.is_match(search, bang) {
                return Some(entries);
            }
        }
        None
    }
}

pub struct GithubRepoPattern {
    rgx: regex::Regex,
}

impl GithubRepoPattern {
    pub fn new() -> Self {
        let rgx = regex::RegexBuilder::new(r"^(\w+)/(\w+)$")
            .build()
            .expect("invalid github repo pattern matcher");
        Self { rgx }
    }
}

impl Pattern for GithubRepoPattern {
    fn is_match(&self, search: &str, _bang: Option<&Bang>) -> Option<Vec<Entry>> {
        let captures = self.rgx.captures(&search);
        match captures {
            None => None,
            Some(c) => {
                let matches: Vec<&str> = c.iter().filter_map(|c| c).map(|c| c.as_str()).collect();
                let owner = matches.get(1).expect("missing repo owner");
                let repo = matches.get(2).expect("missing repo name");
                let name = format!("Github Repo - {owner}/{repo}");
                let search = format!("https://github.com/{owner}/{repo}");
                let action = format!("xdg-open {search:?}");
                let entry = Entry::new(&name, &action, None);
                Some(vec![entry])
            }
        }
    }
}
