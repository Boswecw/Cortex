use crate::db::SearchResult;
use crate::error::Result;

pub struct FtsSearcher {
    // TODO: Implement FTS searcher
}

impl FtsSearcher {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // TODO: Implement FTS search
        Ok(vec![])
    }
}
