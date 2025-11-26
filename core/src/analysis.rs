use crate::pdf::{PdfMutationRequest, PdfMutator, RealPdfMutator};
use crate::pipeline::{LoggingConfig, MetricSpec, PipelineConfig, PipelineType};
use crate::attacks::templates::InjectionTemplate;
use crate::{Result, AnalysisError};
use crate::attacks::ProfileConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Plan for a single analysis step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalysisPlan {
    /// The profile configuration to use.
    pub profile: ProfileConfig,
    /// The ID of the template to use.
    pub template_id: String,
}

/// Defines a complete analysis scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisScenario {
    /// Unique ID for the scenario.
    pub scenario_id: String,
    /// Path to the base PDF file.
    pub base_pdf: PathBuf,
    /// List of analysis plans to perform.
    pub plans: Vec<AnalysisPlan>,
    /// Configuration for the evaluation pipeline.
    pub pipeline: PipelineConfig,
    /// List of metrics to track.
    pub metrics: Vec<MetricSpec>,
    /// Logging configuration.
    pub logging: Option<LoggingConfig>,
}

/// Represents a generated PDF variant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PdfVariant {
    /// Unique ID of the variant.
    pub variant_id: String,
    /// List of profile IDs applied.
    pub profiles: Vec<String>,
    /// List of template IDs applied.
    pub templates: Vec<String>,
    /// Path to the base PDF.
    pub base_pdf: PathBuf,
    /// Path to the mutated PDF.
    pub mutated_pdf: Option<PathBuf>,
    /// Hash of the variant.
    pub variant_hash: Option<String>,
}

/// The impact of a variant on the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantImpact {
    /// ID of the variant.
    pub variant_id: String,
    /// Score before injection.
    pub score_before: Option<f64>,
    /// Score after injection.
    pub score_after: Option<f64>,
    /// Classification label before injection.
    pub classification_before: Option<String>,
    /// Classification label after injection.
    pub classification_after: Option<String>,
    /// Sample response from the LLM.
    pub llm_response_sample: Option<String>,
    /// Profiles applied.
    pub profiles: Vec<String>,
    /// Templates applied.
    pub templates: Vec<String>,
    /// Path to the mutated PDF.
    pub mutated_pdf: Option<PathBuf>,
    /// Hash of the variant.
    pub variant_hash: Option<String>,
    /// Notes or logs.
    pub notes: Vec<String>,
}

/// Report for a full scenario execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioReport {
    /// ID of the scenario.
    pub scenario_id: String,
    /// Target pipeline name.
    pub target: Option<String>,
    /// List of impacts for each variant.
    pub variants: Vec<VariantImpact>,
}

/// The main engine for running Analysis scenarios.
pub struct AnalysisEngine {
    templates: HashMap<String, InjectionTemplate>,
}

impl AnalysisEngine {
    /// Creates a new `AnalysisEngine` with the provided templates.
    pub fn new(templates: impl IntoIterator<Item = InjectionTemplate>) -> Self {
        let map = templates
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect::<HashMap<_, _>>();
        AnalysisEngine { templates: map }
    }

    fn template(&self, id: &str) -> Result<&InjectionTemplate> {
        self.templates
            .get(id)
            .ok_or_else(|| AnalysisError::MissingTemplate(id.to_string()))
    }

    fn build_variant_id(profile: &ProfileConfig, template: &InjectionTemplate) -> String {
        format!("{}_{}", profile.id(), template.id.replace('.', "_"))
    }

    /// Runs a scenario with a specific mutator and pipeline executor.
    pub fn run_with(
        &self,
        scenario: &AnalysisScenario,
        mutator: &dyn PdfMutator,
        pipeline: &dyn PipelineExecutor,
    ) -> Result<ScenarioReport> {
        if scenario.plans.is_empty() {
            return Err(AnalysisError::InvalidScenario(
                "scenario requires at least one plan".into(),
            ));
        }

        let mut impacts = Vec::new();
        for plan in &scenario.plans {
            let template = self.template(&plan.template_id)?;
            let variant_id = Self::build_variant_id(&plan.profile, template);

            let mutation = mutator.mutate(PdfMutationRequest {
                base_pdf: scenario.base_pdf.clone(),
                profiles: vec![plan.profile.clone()],
                template: template.clone(),
                variant_id: Some(variant_id.clone()),
            })?;

            let variant = PdfVariant {
                variant_id: mutation.variant_id.clone(),
                profiles: vec![plan.profile.id().to_string()],
                templates: vec![template.id.clone()],
                base_pdf: scenario.base_pdf.clone(),
                mutated_pdf: Some(mutation.mutated_pdf.clone()),
                variant_hash: mutation.variant_hash.clone(),
            };

            let mut impact = pipeline.evaluate(variant.clone(), scenario)?;
            if impact.mutated_pdf.is_none() {
                impact.mutated_pdf = variant.mutated_pdf.clone();
            }
            if impact.variant_hash.is_none() {
                impact.variant_hash = variant.variant_hash.clone();
            }
            if impact.profiles.is_empty() {
                impact.profiles = variant.profiles.clone();
            }
            if impact.templates.is_empty() {
                impact.templates = variant.templates.clone();
            }

            impacts.push(impact);
        }

        Ok(ScenarioReport {
            scenario_id: scenario.scenario_id.clone(),
            target: scenario.pipeline.target().map(|t| t.to_string()),
            variants: impacts,
        })
    }

    /// Runs a scenario using the real mutator and appropriate pipeline executor.
    pub fn run_scenario(&self, scenario: &AnalysisScenario) -> Result<ScenarioReport> {
        let mutator = RealPdfMutator::new("target/variants");
        match scenario.pipeline.pipeline_type {
            PipelineType::HttpLlm { .. } => {
                let pipeline = HttpPipelineExecutor::new();
                self.run_with(scenario, &mutator, &pipeline)
            }
            PipelineType::LocalPrompt { .. } => {
                let pipeline = LocalPipelineExecutor::new();
                self.run_with(scenario, &mutator, &pipeline)
            }
        }
    }
}

/// Trait for executing the evaluation pipeline.
pub trait PipelineExecutor {
    /// Evaluates a PDF variant against the scenario.
    fn evaluate(
        &self,
        variant: PdfVariant,
        scenario: &AnalysisScenario,
    ) -> Result<VariantImpact>;
}

/// Placeholder pipeline executor that leaves scoring/classification empty but
/// threads through artifact metadata.
pub struct NoopPipelineExecutor;

impl PipelineExecutor for NoopPipelineExecutor {
    fn evaluate(
        &self,
        variant: PdfVariant,
        _scenario: &AnalysisScenario,
    ) -> Result<VariantImpact> {
        Ok(VariantImpact {
            variant_id: variant.variant_id,
            score_before: None,
            score_after: None,
            classification_before: None,
            classification_after: None,
            llm_response_sample: None,
            profiles: variant.profiles,
            templates: variant.templates,
            mutated_pdf: variant.mutated_pdf,
            variant_hash: variant.variant_hash,
            notes: vec!["pipeline execution skipped (noop executor)".into()],
        })
    }
}

/// Pipeline executor that sends requests to an HTTP endpoint.
pub struct HttpPipelineExecutor {
    client: reqwest::blocking::Client,
}

impl HttpPipelineExecutor {
    /// Creates a new HttpPipelineExecutor.
    pub fn new() -> Self {
        HttpPipelineExecutor {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl PipelineExecutor for HttpPipelineExecutor {
    fn evaluate(
        &self,
        variant: PdfVariant,
        scenario: &AnalysisScenario,
    ) -> Result<VariantImpact> {
        match &scenario.pipeline.pipeline_type {
            PipelineType::HttpLlm { endpoint, .. } => {
                // If the endpoint is the example one, skip execution to avoid errors
                if endpoint.contains("example-ats-llm") {
                     return Ok(VariantImpact {
                        variant_id: variant.variant_id,
                        score_before: None,
                        score_after: None,
                        classification_before: None,
                        classification_after: None,
                        llm_response_sample: None,
                        profiles: variant.profiles,
                        templates: variant.templates,
                        mutated_pdf: variant.mutated_pdf,
                        variant_hash: variant.variant_hash,
                        notes: vec!["HttpPipelineExecutor: Skipped example endpoint".into()],
                    });
                }

                // Prepare the request
                let file_path = variant.mutated_pdf.as_ref()
                    .ok_or_else(|| crate::AnalysisError::InvalidScenario("Missing mutated PDF path".into()))?;
                
                let form = reqwest::blocking::multipart::Form::new()
                    .file("file", file_path)
                    .map_err(|e| crate::AnalysisError::Io(e))?;

                let response = self.client.post(endpoint)
                    .multipart(form)
                    .send()
                    .map_err(|e| crate::AnalysisError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

                let status = response.status();
                let text = response.text().unwrap_or_default();

                Ok(VariantImpact {
                    variant_id: variant.variant_id,
                    score_before: None,
                    score_after: None,
                    classification_before: None,
                    classification_after: None,
                    llm_response_sample: Some(text),
                    profiles: variant.profiles,
                    templates: variant.templates,
                    mutated_pdf: variant.mutated_pdf,
                    variant_hash: variant.variant_hash,
                    notes: vec![format!("HttpPipelineExecutor: POST {} -> {}", endpoint, status)],
                })
            }
            _ => {
                // Fallback to no-op
                 Ok(VariantImpact {
                    variant_id: variant.variant_id,
                    score_before: None,
                    score_after: None,
                    classification_before: None,
                    classification_after: None,
                    llm_response_sample: None,
                    profiles: variant.profiles,
                    templates: variant.templates,
                    mutated_pdf: variant.mutated_pdf,
                    variant_hash: variant.variant_hash,
                    notes: vec!["HttpPipelineExecutor: Unsupported pipeline type".into()],
                })
            }
        }
    }
}

/// Pipeline executor that runs locally (extracts text and simulates ATS).
pub struct LocalPipelineExecutor;

impl LocalPipelineExecutor {
    /// Creates a new LocalPipelineExecutor.
    pub fn new() -> Self {
        LocalPipelineExecutor
    }
}

impl PipelineExecutor for LocalPipelineExecutor {
    fn evaluate(
        &self,
        variant: PdfVariant,
        _scenario: &AnalysisScenario,
    ) -> Result<VariantImpact> {
        let file_path = variant.mutated_pdf.as_ref()
            .ok_or_else(|| crate::AnalysisError::InvalidScenario("Missing mutated PDF path".into()))?;

        // Extract text
        let extracted_text = crate::pdf_utils::extract_text_from_pdf(file_path)?;

        // Simple keyword scoring (Simulation)
        let keywords = ["Rust", "Senior", "Engineer", "Leadership", "Expert"];
        let mut score = 0.0;
        let mut found_keywords = Vec::new();

        for keyword in keywords {
            if extracted_text.contains(keyword) {
                score += 10.0;
                found_keywords.push(keyword);
            }
        }

        // Check for injection phrases
        let injection_detected = extracted_text.contains("Ignore previous") 
            || extracted_text.contains("IMPORTANT SYSTEM NOTE")
            || extracted_text.contains("INTERNAL EVALUATION DIRECTIVE")
            || extracted_text.contains("Note to the automated reviewer");

        let notes = vec![
            format!("Extracted {} chars", extracted_text.len()),
            format!("Found keywords: {:?}", found_keywords),
            format!("Injection detected: {}", injection_detected),
        ];

        Ok(VariantImpact {
            variant_id: variant.variant_id,
            score_before: Some(50.0), // Baseline placeholder
            score_after: Some(50.0 + score),
            classification_before: Some("Candidate".into()),
            classification_after: Some(if score > 30.0 { "Top Candidate".into() } else { "Candidate".into() }),
            llm_response_sample: Some(extracted_text.chars().take(200).collect::<String>() + "..."),
            profiles: variant.profiles,
            templates: variant.templates,
            mutated_pdf: variant.mutated_pdf,
            variant_hash: variant.variant_hash,
            notes,
        })
    }
}
