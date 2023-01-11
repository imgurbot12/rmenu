/*
 *   Result Paginator Implementation
 */
use std::cmp::min;

use rmenu_plugin::Entry;

/// Plugin results paginator implementation
pub struct Paginator {
    page: usize,
    page_size: usize,
    results: Vec<Entry>,
    focus: usize,
}

impl Paginator {
    pub fn new(page_size: usize) -> Self {
        Self {
            page: 0,
            page_size,
            results: vec![],
            focus: 0,
        }
    }

    #[inline(always)]
    fn lower_bound(&self) -> usize {
        self.page * self.page_size
    }

    #[inline(always)]
    fn upper_bound(&self) -> usize {
        (self.page + 1) * self.page_size
    }

    fn set_focus(&mut self, focus: usize) {
        self.focus = focus;
        if self.focus < self.lower_bound() {
            self.page -= 1;
        }
        if self.focus >= self.upper_bound() {
            self.page += 1;
        }
    }

    /// reset paginator location and replace internal results
    pub fn reset(&mut self, results: Vec<Entry>) {
        self.page = 0;
        self.focus = 0;
        self.results = results;
    }

    /// calculate zeroed focus based on index in iterator
    #[inline]
    pub fn row_focus(&self) -> usize {
        self.focus - self.lower_bound()
    }

    /// shift focus up a certain number of rows
    #[inline]
    pub fn focus_up(&mut self, shift: usize) {
        self.set_focus(self.focus - min(shift, self.focus));
    }

    /// shift focus down a certain number of rows
    pub fn focus_down(&mut self, shift: usize) {
        let results = self.results.len();
        let max_pos = if results > 0 { results - 1 } else { 0 };
        self.set_focus(min(self.focus + shift, max_pos));
    }

    /// Generate page-size iterator
    #[inline]
    pub fn iter(&self) -> PageIter {
        PageIter::new(self.lower_bound(), self.upper_bound(), &self.results)
    }
}

/// Paginator bounds iterator implementation
pub struct PageIter<'a> {
    stop: usize,
    cursor: usize,
    results: &'a Vec<Entry>,
}

impl<'a> PageIter<'a> {
    pub fn new(start: usize, stop: usize, results: &'a Vec<Entry>) -> Self {
        Self {
            stop,
            results,
            cursor: start,
        }
    }
}

impl<'a> Iterator for PageIter<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.stop {
            return None;
        }
        let result = self.results.get(self.cursor);
        self.cursor += 1;
        result
    }
}
