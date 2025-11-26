use std::process::Command;

/// Manages the LaTeX environment and binary detection.
pub struct LatexManager;

impl LatexManager {
    /// Checks if the configured LaTeX binary is available and executable.
    ///
    /// # Arguments
    ///
    /// * `binary_path` - The path to the LaTeX binary (e.g., "pdflatex").
    ///
    /// # Returns
    ///
    /// * `bool` - True if the binary exists and can be executed.
    pub fn check_binary(binary_path: &str) -> bool {
        Command::new(binary_path)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Attempts to auto-detect the LaTeX binary path from the system PATH.
    ///
    /// # Returns
    ///
    /// * `Option<String>` - The path to the binary if found, otherwise None.
    pub fn auto_detect() -> Option<String> {
        let common_names = ["pdflatex", "xelatex", "lualatex", "tectonic"];
        for name in common_names {
            if Self::check_binary(name) {
                return Some(name.to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_binary_invalid() {
        assert!(!LatexManager::check_binary("non_existent_binary_xyz"));
    }
}
