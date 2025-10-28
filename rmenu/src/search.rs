//! RMENU Entry Search Function Implementaton
use regex::RegexBuilder;
use rmenu_plugin::{Entry, Search};

/// Search through specified entries and return matches in preference
/// of name matches before comment matches
pub fn filter(search: &Search, entries: &Vec<Entry>) -> Vec<Entry> {
    let filter = new_searchfn(search);
    let mut matched: Vec<(&Entry, u8)> = entries
        .iter()
        .map(|e| (e, filter(e)))
        .filter(|(_, v)| v > &0)
        .collect();
    matched.sort_by_key(|(_, v)| *v);
    matched.into_iter().map(|(e, _)| e).cloned().collect()
}

/// Generate a new dynamic Search Function based on
/// Configurtaion Settings and Search-String
#[inline]
fn new_searchfn(search: &Search) -> Box<dyn Fn(&Entry) -> u8> {
    // build regex search expression
    if search.is_regex {
        let rgx = RegexBuilder::new(&search.search)
            .case_insensitive(search.ignore_case)
            .build();
        let Ok(regex) = rgx else {
            return Box::new(|_| 0);
        };
        return Box::new(move |entry: &Entry| {
            if regex.is_match(&entry.name) {
                return 1;
            }
            if let Some(comment) = entry.comment.as_ref() {
                if regex.is_match(&comment) {
                    return 2;
                }
            }
            0
        });
    }
    // build case-insensitive search expression
    if search.ignore_case {
        let matchstr = search.search.to_lowercase();
        return Box::new(move |entry: &Entry| {
            if entry.name.to_lowercase().contains(&matchstr) {
                return 1;
            }
            if let Some(comment) = entry.comment.as_ref() {
                if comment.to_lowercase().contains(&matchstr) {
                    return 2;
                }
            }
            0
        });
    }
    // build standard normal string comparison function
    let matchstr = search.search.to_owned();
    Box::new(move |entry: &Entry| {
        if entry.name.contains(&matchstr) {
            return 1;
        }
        if let Some(comment) = entry.comment.as_ref() {
            if comment.contains(&matchstr) {
                return 2;
            }
        }
        0
    })
}
