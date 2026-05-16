use super::{Segment, SegmentData};
use crate::config::{InputData, ModelConfig, SegmentId};
use crate::core::transcript;
use std::collections::HashMap;

#[derive(Default)]
pub struct ContextWindowSegment;

impl ContextWindowSegment {
    pub fn new() -> Self {
        Self
    }

    /// Get context limit for the specified model
    fn get_context_limit_for_model(model_id: &str) -> u32 {
        let model_config = ModelConfig::load();
        model_config.get_context_limit(model_id)
    }
}

/// Format a token count as "Nk" (rounded to one decimal when fractional)
fn format_k(tokens: u32) -> String {
    if tokens >= 1000 {
        let k = tokens as f64 / 1000.0;
        if k.fract() == 0.0 {
            format!("{}k", k as u32)
        } else {
            format!("{:.1}k", k)
        }
    } else {
        tokens.to_string()
    }
}

/// Pick a c16 ANSI color index that fades from the segment default toward red
/// as remaining-context shrinks. Returns None at >50% (no override → use the
/// user's configured text color).
fn threshold_color(remaining_percent: f64) -> Option<u8> {
    if remaining_percent > 50.0 {
        None
    } else if remaining_percent > 20.0 {
        Some(11) // bright yellow
    } else if remaining_percent > 10.0 {
        Some(9) // bright red
    } else {
        Some(1) // red
    }
}

impl Segment for ContextWindowSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        // Dynamically determine context limit based on current model ID
        let context_limit = Self::get_context_limit_for_model(&input.model.id);

        let context_used_token_opt = transcript::parse_last_assistant_usage(&input.transcript_path)
            .map(|u| u.display_tokens());

        let limit_display = format_k(context_limit);

        let (primary, remaining_percent_opt) = match context_used_token_opt {
            Some(used) => {
                let used = used.min(context_limit);
                let remaining = context_limit.saturating_sub(used);
                let remaining_percent = (remaining as f64 / context_limit as f64) * 100.0;
                (
                    format!("{} / {} tokens", format_k(remaining), limit_display),
                    Some(remaining_percent),
                )
            }
            None => (format!("? / {} tokens", limit_display), None),
        };

        let mut metadata = HashMap::new();
        match (context_used_token_opt, remaining_percent_opt) {
            (Some(used), Some(remaining_percent)) => {
                metadata.insert("tokens".to_string(), used.to_string());
                metadata.insert("percentage".to_string(), remaining_percent.to_string());
                if let Some(c16) = threshold_color(remaining_percent) {
                    metadata.insert("dynamic_text_color".to_string(), c16.to_string());
                }
            }
            _ => {
                metadata.insert("tokens".to_string(), "-".to_string());
                metadata.insert("percentage".to_string(), "-".to_string());
            }
        }
        metadata.insert("limit".to_string(), context_limit.to_string());
        metadata.insert("model".to_string(), input.model.id.clone());

        Some(SegmentData {
            primary,
            secondary: String::new(),
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::ContextWindow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_k_zero() {
        assert_eq!(format_k(0), "0");
    }

    #[test]
    fn format_k_sub_thousand() {
        assert_eq!(format_k(999), "999");
    }

    #[test]
    fn format_k_round_thousand() {
        assert_eq!(format_k(32000), "32k");
    }

    #[test]
    fn format_k_fractional_thousand() {
        assert_eq!(format_k(32500), "32.5k");
    }

    #[test]
    fn threshold_color_default_when_plenty_remaining() {
        assert_eq!(threshold_color(100.0), None);
        assert_eq!(threshold_color(75.0), None);
        assert_eq!(threshold_color(50.5), None);
    }

    #[test]
    fn threshold_color_yellow_band() {
        assert_eq!(threshold_color(50.0), Some(11));
        assert_eq!(threshold_color(30.0), Some(11));
        assert_eq!(threshold_color(20.5), Some(11));
    }

    #[test]
    fn threshold_color_bright_red_band() {
        assert_eq!(threshold_color(20.0), Some(9));
        assert_eq!(threshold_color(15.0), Some(9));
        assert_eq!(threshold_color(10.5), Some(9));
    }

    #[test]
    fn threshold_color_red_band() {
        assert_eq!(threshold_color(10.0), Some(1));
        assert_eq!(threshold_color(5.0), Some(1));
        assert_eq!(threshold_color(0.0), Some(1));
    }
}
