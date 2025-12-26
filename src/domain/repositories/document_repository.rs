use crate::domain::entities::document_content::DocumentContent;
use anyhow::Result;

/// Repository for fetching document contents
#[allow(dead_code)]
pub trait DocumentRepository {
    /// Fetches documents matching the specified glob patterns
    fn fetch_documents(&self, patterns: &[String]) -> Result<Vec<DocumentContent>>;
}
