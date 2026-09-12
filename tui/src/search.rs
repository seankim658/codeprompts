//! The fuzzy file finder.

use crate::prompt_editor::{KeyOutcome, Mode, PromptEditor};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::path::PathBuf;

/// A single searchable entry.
#[derive(Debug, Clone)]
pub struct Candidate {
    /// Path relative to the tree root
    pub rel_path: PathBuf,
    /// Whether this entry is a directory
    pub is_dir: bool,
    /// The string matched against and shown inthe results list
    pub display: String,
}

/// A candidate that matched the current query, with its fuzzy score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Match {
    /// Index into [`FileSearch::candidates`]
    candidate_index: usize,
    /// Fuzzy score
    score: u32,
}

/// How a result should be marked in the file tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkKind {
    /// Toggle the include mark
    Include,
    /// Toggle the exclude mark
    Exclude,
    /// Cycle none -> include -> exclude -> none
    Cycle,
}

/// An aaction the finder needs its caller to perform, because it
/// reaches beyond the finder's own state (the file tree, or closing
/// the modal).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchAction {
    /// Apply `kind` to `path` in the file tree, modal stays open
    Mark { path: PathBuf, kind: MarkKind },
    /// Reveal `path` in the file tree and close the modal
    Reveal(PathBuf),
    /// Close the modal
    Close,
}

/// The fuzzy file finder's state.
pub struct FileSearch {
    /// Whether the modal is currently open
    active: bool,
    /// The query line editor
    editor: PromptEditor,
    /// Every searchable entry, in the order supplied at open
    candidates: Vec<Candidate>,
    /// Candidates matching the current query, best first
    matches: Vec<Match>,
    /// Index into `matches` of the highlighted result
    selected: usize,
    /// The fuzzy matcher, isolating the ranking library
    ranker: Ranker,
}

impl Default for FileSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSearch {
    pub fn new() -> Self {
        Self {
            active: false,
            editor: PromptEditor::new(),
            candidates: Vec::new(),
            matches: Vec::new(),
            selected: 0,
            ranker: Ranker::new(),
        }
    }

    pub fn with_escape_sequence(escape_sequence: Vec<char>) -> Self {
        Self {
            editor: PromptEditor::new().with_escape_sequence(escape_sequence),
            ..Self::new()
        }
    }

    /// Open the finder with a fresh candidate set, resetting the query.
    pub fn open(&mut self, candidates: Vec<(PathBuf, bool)>) {
        self.candidates = candidates
            .into_iter()
            .map(|(rel_path, is_dir)| {
                let display = rel_path.to_string_lossy().into_owned();
                Candidate {
                    rel_path,
                    is_dir,
                    display,
                }
            })
            .collect();
        self.editor.reset();
        self.active = true;
        self.rerank();
    }

    /// Close the finder and release the candidate set.
    pub fn close(&mut self) {
        self.active = false;
        self.candidates.clear();
        self.matches.clear();
        self.selected = 0;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Mutable access to the query editor.
    pub fn editor_mut(&mut self) -> &mut PromptEditor {
        &mut self.editor
    }

    /// Recomputes the match list from the current query and resets the result
    /// cursor to the top.
    pub fn rerank(&mut self) {
        self.matches = self.ranker.rank(&self.editor.text(), &self.candidates);
        self.selected = 0;
    }

    pub fn move_selection_down(&mut self, count: usize) {
        if self.matches.is_empty() {
            return;
        }
        self.selected = (self.selected + count).min(self.matches.len() - 1);
    }

    pub fn move_selection_up(&mut self, count: usize) {
        self.selected = self.selected.saturating_sub(count);
    }

    pub fn selected_candidate(&self) -> Option<&Candidate> {
        self.matches
            .get(self.selected)
            .map(|m| &self.candidates[m.candidate_index])
    }

    pub fn query(&self) -> String {
        self.editor.text()
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn results(&self) -> impl Iterator<Item = &Candidate> + '_ {
        self.matches
            .iter()
            .map(move |m| &self.candidates[m.candidate_index])
    }

    pub fn cursor(&self) -> usize {
        self.editor.cursor()
    }

    pub fn mode(&self) -> Mode {
        self.editor.mode()
    }

    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Interprets one key while the modal is open, returning the action the
    /// caller must perform, or `None` when the finder handled the key itself.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<SearchAction> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Chords that act in both insert and normal mode
        match key.code {
            KeyCode::Char('c') if ctrl => return Some(SearchAction::Close),
            KeyCode::Enter => return self.mark_selected(MarkKind::Cycle),
            KeyCode::Tab => return self.reveal_selected(),
            KeyCode::Char('n') if ctrl => {
                self.move_selection_down(1);
                return None;
            }
            KeyCode::Char('p') if ctrl => {
                self.move_selection_up(1);
                return None;
            }
            KeyCode::Down => {
                self.move_selection_down(1);
                return None;
            }
            KeyCode::Up => {
                self.move_selection_up(1);
                return None;
            }
            _ => {}
        }

        // Query editor
        match self.editor.handle_key(key) {
            KeyOutcome::TextChanged => {
                self.rerank();
                None
            }
            KeyOutcome::Consumed => None,
            KeyOutcome::Ignored => self.handle_normal_command(key),
        }
    }

    /// Handles a normal mode key the editor did not claim
    fn handle_normal_command(&mut self, key: KeyEvent) -> Option<SearchAction> {
        match key.code {
            KeyCode::Char('+') => self.mark_selected(MarkKind::Include),
            KeyCode::Char('-') => self.mark_selected(MarkKind::Exclude),
            KeyCode::Char('q') | KeyCode::Esc => Some(SearchAction::Close),
            KeyCode::Char('j') => {
                self.move_selection_down(1);
                None
            }
            KeyCode::Char('k') => {
                self.move_selection_up(1);
                None
            }
            _ => None,
        }
    }

    /// The mark action for the selected result, or `None` when nothing matches
    fn mark_selected(&self, kind: MarkKind) -> Option<SearchAction> {
        self.selected_candidate().map(|cand| SearchAction::Mark {
            path: cand.rel_path.clone(),
            kind,
        })
    }

    /// The reveal action for the selected result, or `None` when nothing matches.
    fn reveal_selected(&self) -> Option<SearchAction> {
        self.selected_candidate()
            .map(|cand| SearchAction::Reveal(cand.rel_path.clone()))
    }
}

struct Ranker {
    matcher: Matcher,
    buf: Vec<char>,
}

impl Ranker {
    fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT.match_paths()),
            buf: Vec::new(),
        }
    }

    /// Scores every canidate against `query`, returning the matches best first.
    fn rank(&mut self, query: &str, candidates: &[Candidate]) -> Vec<Match> {
        if query.is_empty() {
            return (0..candidates.len())
                .map(|i| Match {
                    candidate_index: i,
                    score: 0,
                })
                .collect();
        }

        let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
        let mut matches: Vec<Match> = candidates
            .iter()
            .enumerate()
            .filter_map(|(i, cand)| {
                let haystack = Utf32Str::new(&cand.display, &mut self.buf);
                pattern
                    .score(haystack, &mut self.matcher)
                    .map(|score| Match {
                        candidate_index: i,
                        score,
                    })
            })
            .collect();

        matches.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.candidate_index.cmp(&b.candidate_index))
        });
        matches
    }
}
