//! Repository filtering logic

use crate::error::{Error, Result};
use crate::types::{FilterCriteria, Repository};
use glob::Pattern;
use regex::Regex;

/// Filter repositories based on criteria
pub fn filter_repositories(repos: Vec<Repository>, criteria: &FilterCriteria) -> Result<Vec<Repository>> {
    let mut filtered = repos;

    // Apply archived filter
    if criteria.exclude_archived {
        filtered.retain(|r| !r.archived);
    }

    // Apply fork filter
    if criteria.exclude_forks {
        filtered.retain(|r| !r.fork);
    }

    // Apply language filter
    if let Some(ref lang) = criteria.language {
        let lang_lower = lang.to_lowercase();
        filtered.retain(|r| {
            r.language
                .as_ref()
                .map(|l| l.to_lowercase() == lang_lower)
                .unwrap_or(false)
        });
    }

    // Apply minimum stars filter
    if let Some(min_stars) = criteria.min_stars {
        filtered.retain(|r| r.stargazers_count >= min_stars);
    }

    // Apply updated_after filter
    if let Some(updated_after) = criteria.updated_after {
        filtered.retain(|r| r.updated_at >= updated_after);
    }

    // Apply include patterns
    if !criteria.include.is_empty() {
        let patterns: Result<Vec<Pattern>> = criteria
            .include
            .iter()
            .map(|p| Pattern::new(p).map_err(|e| Error::InvalidFilter(e.to_string())))
            .collect();
        let patterns = patterns?;

        filtered.retain(|r| {
            patterns.iter().any(|p| p.matches(&r.name) || p.matches(&r.full_name))
        });
    }

    // Apply exclude patterns
    if !criteria.exclude.is_empty() {
        let patterns: Result<Vec<Pattern>> = criteria
            .exclude
            .iter()
            .map(|p| Pattern::new(p).map_err(|e| Error::InvalidFilter(e.to_string())))
            .collect();
        let patterns = patterns?;

        filtered.retain(|r| {
            !patterns.iter().any(|p| p.matches(&r.name) || p.matches(&r.full_name))
        });
    }

    Ok(filtered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_repo(name: &str, archived: bool, fork: bool, stars: u32) -> Repository {
        Repository {
            name: name.to_string(),
            full_name: format!("org/{}", name),
            description: None,
            clone_url: format!("https://github.com/org/{}", name),
            ssh_url: format!("git@github.com:org/{}.git", name),
            default_branch: "main".to_string(),
            private: false,
            fork,
            archived,
            language: Some("Rust".to_string()),
            stargazers_count: stars,
            forks_count: 0,
            updated_at: Utc::now(),
            pushed_at: Utc::now(),
            topics: vec![],
        }
    }

    #[test]
    fn test_filter_archived() {
        let repos = vec![
            create_test_repo("repo1", false, false, 10),
            create_test_repo("repo2", true, false, 20),
        ];

        let criteria = FilterCriteria {
            exclude_archived: true,
            ..Default::default()
        };

        let filtered = filter_repositories(repos, &criteria).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");
    }

    #[test]
    fn test_filter_forks() {
        let repos = vec![
            create_test_repo("repo1", false, false, 10),
            create_test_repo("repo2", false, true, 20),
        ];

        let criteria = FilterCriteria {
            exclude_forks: true,
            ..Default::default()
        };

        let filtered = filter_repositories(repos, &criteria).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");
    }

    #[test]
    fn test_filter_min_stars() {
        let repos = vec![
            create_test_repo("repo1", false, false, 10),
            create_test_repo("repo2", false, false, 20),
        ];

        let criteria = FilterCriteria {
            min_stars: Some(15),
            ..Default::default()
        };

        let filtered = filter_repositories(repos, &criteria).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo2");
    }

    #[test]
    fn test_filter_include_pattern() {
        let repos = vec![
            create_test_repo("hack-tool", false, false, 10),
            create_test_repo("other-repo", false, false, 20),
        ];

        let criteria = FilterCriteria {
            include: vec!["hack-*".to_string()],
            ..Default::default()
        };

        let filtered = filter_repositories(repos, &criteria).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "hack-tool");
    }

    #[test]
    fn test_filter_exclude_pattern() {
        let repos = vec![
            create_test_repo("hack-tool", false, false, 10),
            create_test_repo("other-repo", false, false, 20),
        ];

        let criteria = FilterCriteria {
            exclude: vec!["hack-*".to_string()],
            ..Default::default()
        };

        let filtered = filter_repositories(repos, &criteria).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "other-repo");
    }
}
