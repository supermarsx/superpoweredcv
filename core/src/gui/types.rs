use std::path::PathBuf;
use crate::attacks::{InjectionPosition, Intensity};
use crate::attacks::templates::GenerationType;

#[derive(PartialEq, Clone)]
pub enum InputSource {
    JsonFile(Option<PathBuf>),
    PdfFile(Option<PathBuf>),
    LinkedinUrl(String),
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Mistral,
    Groq,
    OpenRouter,
    LocalAI,
    Ollama,
    LMStudio,
    Gemini,
    Cohere,
    DeepSeek,
    Custom,
}

#[derive(Clone)]
pub struct InjectionConfigGui {
    pub injection_type: InjectionTypeGui,
    pub intensity: Intensity,
    pub position: InjectionPosition,
    pub phrases: Vec<String>,
    pub current_phrase: String,
    pub generation_type: GenerationType,
    pub job_description: String,
}

impl Default for InjectionConfigGui {
    fn default() -> Self {
        Self {
            injection_type: InjectionTypeGui::VisibleMetaBlock,
            intensity: Intensity::Medium,
            position: InjectionPosition::Header,
            phrases: vec![],
            current_phrase: String::new(),
            generation_type: GenerationType::Static,
            job_description: String::new(),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InjectionTypeGui {
    VisibleMetaBlock,
    LowVisibilityBlock,
    OffpageLayer,
    UnderlayText,
    StructuralFields,
    PaddingNoise,
    InlineJobAd,
    TrackingPixel,
    CodeInjection,
}

#[derive(Default, Clone)]
pub struct ProfileMask {
    pub experience_enabled: Vec<bool>,
    pub education_enabled: Vec<bool>,
    pub skills_enabled: Vec<bool>,
}
