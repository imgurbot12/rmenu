//! RMENU Entry Search Function Implementaton
use regex::RegexBuilder;
use rmenu_plugin::{Entry, Search};

/// Generate a new dynamic Search Function based on
/// Configurtaion Settings and Search-String
pub fn new_searchfn(search: &Search) -> Box<dyn Fn(&Entry) -> bool> {
    // build regex search expression
    if search.is_regex {
        let rgx = RegexBuilder::new(&search.search)
            .case_insensitive(search.ignore_case)
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
    if search.ignore_case {
        let matchstr = search.search.to_lowercase();
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
    let matchstr = search.search.to_owned();
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
