use serde::{Deserialize, Serialize};

/// Represents a document content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DocumentContent {
    file_path: String,
    content: String,
}

impl DocumentContent {
    /// Creates a new DocumentContent instance
    #[allow(dead_code)]
    pub fn new(file_path: String, content: String) -> Self {
        Self { file_path, content }
    }

    /// Returns the file path
    #[allow(dead_code)]
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Returns the content
    #[allow(dead_code)]
    pub fn content(&self) -> &str {
        &self.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn ドキュメントを作成できる() {
        let doc = DocumentContent::new(
            "docs/README.md".to_string(),
            "# Title\n\nContent here".to_string(),
        );

        assert_eq!(doc.file_path(), "docs/README.md");
        assert_eq!(doc.content(), "# Title\n\nContent here");
    }

    #[test]
    #[allow(non_snake_case)]
    fn 空のコンテンツでドキュメントを作成できる() {
        let doc = DocumentContent::new("empty.md".to_string(), String::new());

        assert_eq!(doc.file_path(), "empty.md");
        assert_eq!(doc.content(), "");
    }
}
