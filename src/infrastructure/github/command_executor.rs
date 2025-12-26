use anyhow::Result;
use std::process::Command;

/// Trait for executing commands
#[allow(dead_code)] // Phase 2: Will be used when integrated into main application
pub trait CommandExecutor {
    /// Executes a command and returns the output
    fn execute(&self, program: &str, args: &[&str]) -> Result<String>;
}

/// Real command executor using std::process::Command
#[allow(dead_code)] // Phase 2: Will be used when integrated into main application
pub struct GhCommandExecutor;

impl GhCommandExecutor {
    #[allow(dead_code)] // Phase 2: Will be used when integrated into main application
    pub fn new() -> Self {
        Self
    }
}

impl CommandExecutor for GhCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(program).args(args).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {}", stderr);
        }

        Ok(String::from_utf8(output.stdout)?)
    }
}

#[cfg(test)]
pub struct MockCommandExecutor {
    responses: std::collections::HashMap<String, String>,
}

#[cfg(test)]
impl MockCommandExecutor {
    pub fn new() -> Self {
        Self {
            responses: std::collections::HashMap::new(),
        }
    }

    pub fn with_response(mut self, command_key: &str, response: &str) -> Self {
        self.responses
            .insert(command_key.to_string(), response.to_string());
        self
    }
}

#[cfg(test)]
impl CommandExecutor for MockCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<String> {
        let key = format!("{} {}", program, args.join(" "));

        // Try exact match first
        if let Some(response) = self.responses.get(&key) {
            return Ok(response.clone());
        }

        // Try prefix match (for queries with variable content)
        for (pattern, response) in &self.responses {
            if key.starts_with(pattern) {
                return Ok(response.clone());
            }
        }

        Err(anyhow::anyhow!("No mock response for: {}", key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn MockCommandExecutorが正しく動作する() {
        let mock = MockCommandExecutor::new().with_response("gh api test", r#"{"data": "test"}"#);

        let result = mock.execute("gh", &["api", "test"]).expect("Failed");
        assert_eq!(result, r#"{"data": "test"}"#);
    }
}
