use regex::RegexBuilder;
use rmenu_plugin::Entry;

use crate::config::Config;

macro_rules! search {
    ($search:expr) => {
        Box::new(move |entry: &Entry| {
            if entry.name.contains($search) {
                return true;
            }
            if let Some(comment) = entry.comment.as_ref() {
                return comment.contains($search);
            }
            false
        })
    };
    ($search:expr,$mod:ident) => {
        Box::new(move |entry: &Entry| {
            if entry.name.$mod().contains($search) {
                return true;
            }
            if let Some(comment) = entry.comment.as_ref() {
                return comment.$mod().contains($search);
            }
            false
        })
    };
}

/// Generate a new dynamic Search Function based on
/// Configurtaion Settigns and Search-String
pub fn new_searchfn(cfg: &Config, search: &str) -> Box<dyn Fn(&Entry) -> bool> {
    if cfg.regex {
        let regex = RegexBuilder::new(search)
            .case_insensitive(cfg.ignore_case)
            .build();
        return match regex {
            Ok(rgx) => search!(&rgx),
            Err(_) => Box::new(|_| false),
        };
    }
    if cfg.ignore_case {
        let matchstr = search.to_lowercase();
        return search!(&matchstr, to_lowercase);
    }
    let matchstr = search.to_owned();
    return search!(&matchstr);
}
