use crate::domain::entities::document_content::DocumentContent;
use crate::domain::repositories::document_repository::DocumentRepository;
use anyhow::{Context, Result};
use glob::glob;
use std::fs;

/// Local file document repository implementation
#[allow(dead_code)]
pub struct LocalFileDocumentRepository;

impl LocalFileDocumentRepository {
    /// Creates a new LocalFileDocumentRepository instance
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

impl DocumentRepository for LocalFileDocumentRepository {
    fn fetch_documents(&self, patterns: &[String]) -> Result<Vec<DocumentContent>> {
        let mut documents = Vec::new();

        for pattern in patterns {
            for entry in
                glob(pattern).context(format!("Failed to parse glob pattern: {}", pattern))?
            {
                let path = entry.context("Failed to read glob entry")?;

                // Skip directories
                if path.is_dir() {
                    continue;
                }

                let file_path = path
                    .to_str()
                    .context("Failed to convert path to string")?
                    .to_string();

                let content = fs::read_to_string(&path)
                    .context(format!("Failed to read file: {}", file_path))?;

                documents.push(DocumentContent::new(file_path, content));
            }
        }

        Ok(documents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    #[allow(non_snake_case)]
    fn Globパターンでドキュメントを取得できる() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create test files
        fs::write(temp_path.join("doc1.md"), "Content 1").expect("Failed to write file");
        fs::write(temp_path.join("doc2.md"), "Content 2").expect("Failed to write file");
        fs::write(temp_path.join("doc3.txt"), "Content 3").expect("Failed to write file");

        let repository = LocalFileDocumentRepository::new();
        let pattern = temp_path.join("*.md").to_str().unwrap().to_string();
        let documents = repository
            .fetch_documents(&[pattern])
            .expect("Failed to fetch documents");

        assert_eq!(documents.len(), 2);

        // Check that all documents are .md files
        for doc in &documents {
            assert!(doc.file_path().ends_with(".md"));
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn 複数のGlobパターンでドキュメントを取得できる() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create subdirectory
        let sub_dir = temp_path.join("sub");
        fs::create_dir_all(&sub_dir).expect("Failed to create sub dir");

        fs::write(temp_path.join("doc1.md"), "Content 1").expect("Failed to write file");
        fs::write(sub_dir.join("doc2.md"), "Content 2").expect("Failed to write file");

        let repository = LocalFileDocumentRepository::new();
        let patterns = vec![
            temp_path.join("*.md").to_str().unwrap().to_string(),
            sub_dir.join("*.md").to_str().unwrap().to_string(),
        ];

        let documents = repository
            .fetch_documents(&patterns)
            .expect("Failed to fetch documents");

        assert_eq!(documents.len(), 2);
    }

    #[test]
    #[allow(non_snake_case)]
    fn パターンにマッチするファイルがない場合は空のVecを返す() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        let repository = LocalFileDocumentRepository::new();
        let pattern = temp_path
            .join("*.nonexistent")
            .to_str()
            .unwrap()
            .to_string();
        let documents = repository
            .fetch_documents(&[pattern])
            .expect("Failed to fetch documents");

        assert_eq!(documents.len(), 0);
    }

    #[test]
    #[allow(non_snake_case)]
    fn ファイルの内容を正しく読み込める() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        let test_content = "# Test Document\n\nThis is a test.";
        fs::write(temp_path.join("test.md"), test_content).expect("Failed to write file");

        let repository = LocalFileDocumentRepository::new();
        let pattern = temp_path.join("*.md").to_str().unwrap().to_string();
        let documents = repository
            .fetch_documents(&[pattern])
            .expect("Failed to fetch documents");

        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].content(), test_content);
    }
}
