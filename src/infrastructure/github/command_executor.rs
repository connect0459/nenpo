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

        // For GraphQL queries, stdout may contain valid JSON even if the command fails
        // (e.g., when querying a non-existent organization but user data is available)
        let stdout = String::from_utf8(output.stdout)?;
        if !output.status.success() && stdout.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {}", stderr);
        }

        Ok(stdout)
    }
}

#[cfg(test)]
pub struct MockCommandExecutor {
    responses: std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>,
    call_count: std::sync::Arc<std::sync::Mutex<usize>>,
}

#[cfg(test)]
impl MockCommandExecutor {
    pub fn new() -> Self {
        Self {
            responses: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    pub fn with_response(self, command_key: &str, response: &str) -> Self {
        self.responses
            .lock()
            .unwrap()
            .push((command_key.to_string(), response.to_string()));
        self
    }
}

#[cfg(test)]
impl CommandExecutor for MockCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<String> {
        let key = format!("{} {}", program, args.join(" "));

        let responses = self.responses.lock().unwrap();
        let mut call_count = self.call_count.lock().unwrap();

        // Try exact match for current call
        for (i, (pattern, response)) in responses.iter().enumerate() {
            if key == *pattern || key.starts_with(pattern) {
                // For multiple responses with the same pattern, return them in order
                if i == *call_count {
                    *call_count += 1;
                    return Ok(response.clone());
                }
            }
        }

        // If no exact match at the call count, try any prefix match
        for (pattern, response) in responses.iter() {
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
