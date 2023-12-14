//! RMENU Entry Search Function Implementaton
use regex::RegexBuilder;
use rmenu_plugin::Entry;

use crate::config::Config;

/// Generate a new dynamic Search Function based on
/// Configurtaion Settings and Search-String
pub fn build_searchfn(cfg: &Config, search: &str) -> Box<dyn Fn(&Entry) -> bool> {
    // build regex search expression
    if cfg.search.use_regex {
        let rgx = RegexBuilder::new(search)
            .case_insensitive(cfg.search.ignore_case)
            .build();
        let Ok(regex) = rgx else {
            return Box::new(|_| false);
        };
        return Box::new(move |entry: &Entry| {
            if regex.is_match(&entry.name) {
                return true;
            }
            if let Some(comment) = entry.comment.as_ref() {
                return regex.is_match(&comment);
            }
            false
        });
    }
    // build case-insensitive search expression
    if cfg.search.ignore_case {
        let matchstr = search.to_lowercase();
        return Box::new(move |entry: &Entry| {
            if entry.name.to_lowercase().contains(&matchstr) {
                return true;
            }
            if let Some(comment) = entry.comment.as_ref() {
                return comment.to_lowercase().contains(&matchstr);
            }
            false
        });
    }
    // build standard normal string comparison function
    let matchstr = search.to_owned();
    Box::new(move |entry: &Entry| {
        if entry.name.contains(&matchstr) {
            return true;
        }
        if let Some(comment) = entry.comment.as_ref() {
            return comment.contains(&matchstr);
        }
        false
    })
}
