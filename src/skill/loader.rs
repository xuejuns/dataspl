use super::{Skill, SkillError, SkillExample};
use regex::Regex;
use std::collections::HashMap;
use tokio::fs;

/// Skill loader - loads skills from markdown files
pub struct SkillLoader;

impl SkillLoader {
    /// Load a skill from a markdown file
    pub async fn load_from_file(path: &str) -> Result<Skill, SkillError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| SkillError::LoadError(format!("Failed to read file {}: {}", path, e)))?;

        Self::parse_markdown(&content, path)
    }

    /// Load all skills from a directory
    pub async fn load_from_directory(dir_path: &str) -> Result<Vec<Skill>, SkillError> {
        let mut skills = Vec::new();
        let mut entries = fs::read_dir(dir_path)
            .await
            .map_err(|e| SkillError::LoadError(format!("Failed to read directory {}: {}", dir_path, e)))?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| SkillError::LoadError(e.to_string()))? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                match Self::load_from_file(path.to_str().unwrap_or("")).await {
                    Ok(skill) => skills.push(skill),
                    Err(e) => tracing::warn!("Failed to load skill from {:?}: {}", path, e),
                }
            }
        }

        Ok(skills)
    }

    /// Parse markdown content into a Skill
    pub fn parse_markdown(content: &str, source: &str) -> Result<Skill, SkillError> {
        // Extract frontmatter (YAML between ---)
        let frontmatter_re = Regex::new(r"(?s)^---\n(.+?)\n---").unwrap();
        let frontmatter_caps = frontmatter_re.captures(content);

        let (id, name, description, version) = if let Some(ref caps) = frontmatter_caps {
            let yaml_content = caps.get(1).unwrap().as_str();
            Self::parse_frontmatter(yaml_content)?
        } else {
            // Fallback: derive from filename
            let id = Self::extract_id_from_source(source);
            (id.clone(), id, "No description".to_string(), "1.0.0".to_string())
        };

        // Extract prompt (content after first ```prompt or just the main content)
        let prompt_re = Regex::new(r"(?s)```prompt\s*\n(.+?)```").unwrap();
        let prompt = if let Some(caps) = prompt_re.captures(content) {
            caps.get(1).unwrap().as_str().trim().to_string()
        } else {
            // Try to extract main content (everything not in code blocks or after frontmatter)
            Self::extract_main_content(content, frontmatter_caps.is_some())
        };

        // Extract examples
        let examples = Self::extract_examples(content);

        Ok(Skill {
            id,
            name,
            description,
            version,
            prompt,
            examples,
        })
    }

    fn parse_frontmatter(yaml: &str) -> Result<(String, String, String, String), SkillError> {
        let mut id = String::new();
        let mut name = String::new();
        let mut description = String::new();
        let mut version = "1.0.0".to_string();

        for line in yaml.lines() {
            let line = line.trim();
            if line.starts_with("id:") {
                id = line.trim_start_matches("id:").trim().to_string();
            } else if line.starts_with("name:") {
                name = line.trim_start_matches("name:").trim().to_string();
            } else if line.starts_with("description:") {
                description = line.trim_start_matches("description:").trim().to_string();
            } else if line.starts_with("version:") {
                version = line.trim_start_matches("version:").trim().to_string();
            }
        }

        if id.is_empty() {
            return Err(SkillError::ParseError("Missing 'id' in frontmatter".to_string()));
        }
        if name.is_empty() {
            name = id.clone();
        }

        Ok((id, name, description, version))
    }

    fn extract_main_content(content: &str, has_frontmatter: bool) -> String {
        let start_pos = if has_frontmatter {
            // Skip frontmatter and find next content
            let re = Regex::new(r"(?s)^---\n.+?\n---\n\n(.+)").unwrap();
            if let Some(caps) = re.captures(content) {
                caps.get(1).unwrap().as_str()
            } else {
                // Try simpler pattern
                let re2 = Regex::new(r"(?s)^---\n.+?\n---\s*\n(.+)").unwrap();
                if let Some(caps) = re2.captures(content) {
                    caps.get(1).unwrap().as_str()
                } else {
                    ""
                }
            }
        } else {
            content
        };

        // Remove markdown headers and clean up
        let re = Regex::new(r"(?m)^[#*`_~\[\]]+.*$").unwrap();
        let cleaned = re.replace_all(start_pos, "");
        let cleaned = cleaned.trim().to_string();

        if cleaned.is_empty() {
            content.to_string()
        } else {
            cleaned
        }
    }

    fn extract_examples(content: &str) -> Vec<SkillExample> {
        let mut examples = Vec::new();
        let example_re = Regex::new(r"(?s)### Example\s*\n\s*(.+?)\n```\s*\n(.+?)```").unwrap();

        for caps in example_re.captures_iter(content) {
            let desc = caps.get(1).unwrap().as_str().trim();
            let input = caps.get(2).unwrap().as_str().trim().to_string();
            examples.push(SkillExample {
                description: desc.to_string(),
                input,
                expected_output: None,
            });
        }

        examples
    }

    fn extract_id_from_source(source: &str) -> String {
        // Extract filename without extension and path
        let filename = std::path::Path::new(source)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        filename.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let yaml = r#"
id: test-skill
name: Test Skill
description: A test skill for testing
version: 1.0.0
"#;
        let result = SkillLoader.parse_frontmatter(yaml).unwrap();
        assert_eq!(result.0, "test-skill");
        assert_eq!(result.1, "Test Skill");
        assert_eq!(result.2, "A test skill for testing");
        assert_eq!(result.3, "1.0.0");
    }

    #[test]
    fn test_parse_markdown_with_frontmatter() {
        let content = r#"---
id: my-skill
name: My Skill
description: Does something useful
---

# My Skill

This skill does something very useful.

```prompt
Analyze the following: {input}
```
"#;

        let skill = SkillLoader::parse_markdown(content, "my-skill.md").unwrap();
        assert_eq!(skill.id, "my-skill");
        assert_eq!(skill.name, "My Skill");
        assert!(skill.prompt.contains("Analyze the following"));
    }
}