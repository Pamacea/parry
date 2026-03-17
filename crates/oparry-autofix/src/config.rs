//! Auto-fix configuration

/// Auto-fix configuration
#[derive(Debug, Clone)]
pub struct AutoFixConfig {
    /// Fix strategy - determines which issues to auto-fix
    pub strategy: FixStrategy,
    /// Dry run mode - preview fixes without applying
    pub dry_run: bool,
    /// Maximum number of fixes per file
    pub max_fixes: usize,
    /// Whether to preserve formatting
    pub preserve_formatting: bool,
}

impl Default for AutoFixConfig {
    fn default() -> Self {
        Self {
            strategy: FixStrategy::Moderate,
            dry_run: false,
            max_fixes: 50,
            preserve_formatting: true,
        }
    }
}

/// Fix strategy - how aggressive to be with auto-fixes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixStrategy {
    /// Safe mode - only fix low-risk issues (notes)
    Safe,
    /// Moderate mode - fix warnings and notes (default)
    Moderate,
    /// Aggressive mode - fix everything including errors (when safe)
    Aggressive,
}

impl FixStrategy {
    /// Get the display name for this strategy
    pub fn name(&self) -> &'static str {
        match self {
            FixStrategy::Safe => "safe",
            FixStrategy::Moderate => "moderate",
            FixStrategy::Aggressive => "aggressive",
        }
    }

    /// Parse strategy from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "safe" => Some(FixStrategy::Safe),
            "moderate" => Some(FixStrategy::Moderate),
            "aggressive" => Some(FixStrategy::Aggressive),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_fix_config_default() {
        let config = AutoFixConfig::default();
        assert_eq!(config.strategy, FixStrategy::Moderate);
        assert!(!config.dry_run);
        assert_eq!(config.max_fixes, 50);
        assert!(config.preserve_formatting);
    }

    #[test]
    fn test_fix_strategy_names() {
        assert_eq!(FixStrategy::Safe.name(), "safe");
        assert_eq!(FixStrategy::Moderate.name(), "moderate");
        assert_eq!(FixStrategy::Aggressive.name(), "aggressive");
    }

    #[test]
    fn test_fix_strategy_from_str() {
        assert_eq!(FixStrategy::from_str("safe"), Some(FixStrategy::Safe));
        assert_eq!(FixStrategy::from_str("MODERATE"), Some(FixStrategy::Moderate));
        assert_eq!(FixStrategy::from_str("Aggressive"), Some(FixStrategy::Aggressive));
        assert_eq!(FixStrategy::from_str("invalid"), None);
    }

    #[test]
    fn test_fix_strategy_round_trip() {
        for strategy in [FixStrategy::Safe, FixStrategy::Moderate, FixStrategy::Aggressive] {
            assert_eq!(FixStrategy::from_str(strategy.name()), Some(strategy));
        }
    }
}
