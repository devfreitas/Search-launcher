use crate::indexer::AppEntry;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub fn search_apps(query: &str, index: &[AppEntry]) -> Vec<AppEntry> {
    if query.is_empty() {
        let mut initial: Vec<AppEntry> = index.iter().take(50).cloned().collect();
        initial.sort_by(|a, b| b.priority.cmp(&a.priority));
        return initial.into_iter().take(10).collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(i64, AppEntry)> = index
        .iter()
        .filter_map(|app| {
            matcher.fuzzy_match(&app.name, query).map(|score| {
                let final_score = score * (app.priority as i64);
                (final_score, app.clone())
            })
        })
        .collect();

    results.sort_by(|a, b| b.0.cmp(&a.0));
    
    results.into_iter().map(|(_, app)| app).take(10).collect()
}