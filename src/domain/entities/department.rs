use serde::{Deserialize, Serialize};

/// Represents a department in the organization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)] // Temporarily allowed during TDD implementation
pub struct Department {
    name: String,
    fiscal_year_start_month: u32,
    github_organizations: Vec<String>,
    local_documents: Vec<String>,
}

impl Department {
    /// Creates a new Department instance
    ///
    /// # Panics
    ///
    /// Panics if fiscal_year_start_month is not between 1 and 12
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn new(
        name: String,
        fiscal_year_start_month: u32,
        github_organizations: Vec<String>,
        local_documents: Vec<String>,
    ) -> Self {
        assert!(
            (1..=12).contains(&fiscal_year_start_month),
            "Fiscal year start month must be between 1 and 12, got {}",
            fiscal_year_start_month
        );

        Self {
            name,
            fiscal_year_start_month,
            github_organizations,
            local_documents,
        }
    }

    /// Returns the name of the department
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the fiscal year start month (1-12)
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn fiscal_year_start_month(&self) -> u32 {
        self.fiscal_year_start_month
    }

    /// Returns the list of GitHub organizations
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn github_organizations(&self) -> &[String] {
        &self.github_organizations
    }

    /// Returns the list of local document glob patterns
    #[allow(dead_code)] // Temporarily allowed during TDD implementation
    pub fn local_documents(&self) -> &[String] {
        &self.local_documents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 部門を作成できる() {
        let department = Department::new(
            "個人".to_string(),
            4,
            vec!["connect0459".to_string()],
            vec![],
        );

        assert_eq!(department.name(), "個人");
        assert_eq!(department.fiscal_year_start_month(), 4);
        assert_eq!(
            department.github_organizations(),
            &vec!["connect0459".to_string()]
        );
        assert_eq!(department.local_documents(), &Vec::<String>::new());
    }

    #[test]
    fn 年度開始月が1から12の範囲外の場合はエラーになる() {
        let result =
            std::panic::catch_unwind(|| Department::new("テスト".to_string(), 0, vec![], vec![]));
        assert!(result.is_err());

        let result =
            std::panic::catch_unwind(|| Department::new("テスト".to_string(), 13, vec![], vec![]));
        assert!(result.is_err());
    }
}
