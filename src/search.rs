use crate::indexer::AppEntry;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub fn search_apps(query: &str, index: &[AppEntry]) -> Vec<AppEntry> {
    if query.is_empty() {
        return index.iter().take(10).cloned().collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(i64, AppEntry)> = index
        .iter()
        .filter_map(|app| {
            matcher.fuzzy_match(&app.name, query).map(|score| (score, app.clone()))
        })
        .collect();

    results.sort_by(|a, b| b.0.cmp(&a.0));
    
    results.into_iter().map(|(_, app)| app).take(10).collect()
}