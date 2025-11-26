use crate::pdf::{PdfMutationRequest, PdfMutator, StubPdfMutator};
use crate::pipeline::{LoggingConfig, MetricSpec, PipelineConfig};
use crate::attacks::templates::InjectionTemplate;
use crate::{Result, SimulationError};
use crate::attacks::{ProfileConfig, InjectionPosition, Intensity, LowVisibilityPalette, OffpageOffset, StructuralTarget, PaddingStyle, JobAdSource, JobAdPlacement};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Plan for a single injection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InjectionPlan {
    /// The profile configuration to use.
    pub profile: ProfileConfig,
    /// The ID of the template to use.
    pub template_id: String,
}

/// Defines a complete injection scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionScenario {
    /// Unique ID for the scenario.
    pub scenario_id: String,
    /// Path to the base PDF file.
    pub base_pdf: PathBuf,
    /// List of injections to perform.
    pub injections: Vec<InjectionPlan>,
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

/// The main engine for running Simulation scenarios.
pub struct SimulationEngine {
    templates: HashMap<String, InjectionTemplate>,
}

impl SimulationEngine {
    /// Creates a new `SimulationEngine` with the provided templates.
    pub fn new(templates: impl IntoIterator<Item = InjectionTemplate>) -> Self {
        let map = templates
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect::<HashMap<_, _>>();
        SimulationEngine { templates: map }
    }

    fn template(&self, id: &str) -> Result<&InjectionTemplate> {
        self.templates
            .get(id)
            .ok_or_else(|| SimulationError::MissingTemplate(id.to_string()))
    }

    fn build_variant_id(profile: &ProfileConfig, template: &InjectionTemplate) -> String {
        format!("{}_{}", profile.id(), template.id.replace('.', "_"))
    }

    /// Runs a scenario with a specific mutator and pipeline executor.
    pub fn run_with(
        &self,
        scenario: &InjectionScenario,
        mutator: &dyn PdfMutator,
        pipeline: &dyn PipelineExecutor,
    ) -> Result<ScenarioReport> {
        if scenario.injections.is_empty() {
            return Err(SimulationError::InvalidScenario(
                "scenario requires at least one injection".into(),
            ));
        }

        let mut impacts = Vec::new();
        for injection in &scenario.injections {
            let template = self.template(&injection.template_id)?;
            let variant_id = Self::build_variant_id(&injection.profile, template);

            let mutation = mutator.mutate(PdfMutationRequest {
                base_pdf: scenario.base_pdf.clone(),
                profiles: vec![injection.profile.clone()],
                template: template.clone(),
                variant_id: Some(variant_id.clone()),
            })?;

            let variant = PdfVariant {
                variant_id: mutation.variant_id.clone(),
                profiles: vec![injection.profile.id().to_string()],
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

    /// Runs a scenario using the default stub mutator and no-op pipeline.
    pub fn run_scenario(&self, scenario: &InjectionScenario) -> Result<ScenarioReport> {
        let mutator = StubPdfMutator::new("target/variants");
        let pipeline = NoopPipelineExecutor;
        self.run_with(scenario, &mutator, &pipeline)
    }
}

/// Trait for executing the evaluation pipeline.
pub trait PipelineExecutor {
    /// Evaluates a PDF variant against the scenario.
    fn evaluate(
        &self,
        variant: PdfVariant,
        scenario: &InjectionScenario,
    ) -> Result<VariantImpact>;
}

/// Placeholder pipeline executor that leaves scoring/classification empty but
/// threads through artifact metadata.
pub struct NoopPipelineExecutor;

impl PipelineExecutor for NoopPipelineExecutor {
    fn evaluate(
        &self,
        variant: PdfVariant,
        _scenario: &InjectionScenario,
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
