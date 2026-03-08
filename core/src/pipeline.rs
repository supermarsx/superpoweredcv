use serde::{Deserialize, Serialize};

/// Defines the type of pipeline to simulate or interact with.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PipelineType {
    /// An HTTP-based LLM endpoint.
    HttpLlm {
        /// The URL of the endpoint.
        endpoint: String,
        /// Path to a prompt template file.
        prompt_template: Option<String>,
    },
    /// A local prompt execution (e.g., using a local model).
    LocalPrompt {
        /// Name of the model to use.
        model: Option<String>,
        /// Path to a prompt template file.
        prompt_template: Option<String>,
    },
}

/// Configuration for the evaluation pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PipelineConfig {
    /// The type of pipeline.
    pub pipeline_type: PipelineType,
    /// The target service or component name.
    pub target: Option<String>,
}

impl PipelineConfig {
    /// Returns the target name as a string slice, if present.
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }
}

/// Types of metrics that can be collected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetricType {
    /// Numerical difference between scores.
    NumericDiff,
    /// Change in classification label.
    LabelChange,
}

/// Specification for a metric to be tracked.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricSpec {
    /// Name of the metric.
    pub name: String,
    /// Type of the metric.
    pub metric_type: MetricType,
    /// Baseline value for comparison.
    pub baseline: Option<f64>,
}

/// Configuration for logging pipeline execution details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoggingConfig {
    /// List of fields to capture in logs.
    pub capture: Vec<LogField>,
}

/// Fields that can be captured in logs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogField {
    /// The raw response from the LLM.
    RawLlmResponse,
    /// The text extracted from the PDF.
    ExtractedText,
    /// The hash of the generated PDF variant.
    PdfVariantHash,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_target() {
        let config_with = PipelineConfig {
            pipeline_type: PipelineType::HttpLlm {
                endpoint: "https://example.com".into(),
                prompt_template: None,
            },
            target: Some("my-ats".into()),
        };
        assert_eq!(config_with.target(), Some("my-ats"));

        let config_without = PipelineConfig {
            pipeline_type: PipelineType::LocalPrompt {
                model: None,
                prompt_template: None,
            },
            target: None,
        };
        assert_eq!(config_without.target(), None);
    }

    #[test]
    fn test_metric_spec_serialization() {
        let specs = vec![
            MetricSpec {
                name: "score_diff".into(),
                metric_type: MetricType::NumericDiff,
                baseline: Some(50.0),
            },
            MetricSpec {
                name: "label_change".into(),
                metric_type: MetricType::LabelChange,
                baseline: None,
            },
        ];

        for spec in &specs {
            let json = serde_json::to_string(spec).expect("serialize MetricSpec");
            let deser: MetricSpec = serde_json::from_str(&json).expect("deserialize MetricSpec");
            assert_eq!(spec, &deser);
        }
    }
}
