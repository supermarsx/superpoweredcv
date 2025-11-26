use serde::{Deserialize, Serialize};

pub mod templates;
use templates::GenerationType;

/// Defines where the injection should be placed in the document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InjectionPosition {
    /// Place in the header.
    Header,
    /// Place in the footer.
    Footer,
    /// Place in a specific named section.
    Section(String),
}

/// Defines the intensity of the injection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Intensity {
    /// Soft intensity.
    Soft,
    /// Medium intensity.
    Medium,
    /// Aggressive intensity.
    Aggressive,
    /// Custom intensity (uses provided content directly).
    Custom,
}

/// Palette for low-visibility text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LowVisibilityPalette {
    /// Gray color.
    Gray,
    /// Light blue color.
    LightBlue,
    /// Off-white color.
    OffWhite,
}

/// Strategy for placing text off-page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OffpageOffset {
    /// Clip at the bottom of the page.
    BottomClip,
    /// Clip at the right of the page.
    RightClip,
}

/// Target for structural injections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StructuralTarget {
    /// Inject into Alt Text.
    AltText,
    /// Inject into PDF Tags.
    PdfTag,
    /// Inject into XMP Metadata.
    XmpMetadata,
}

/// Style of padding noise.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaddingStyle {
    /// Padding that looks like resume content.
    ResumeLike,
    /// Padding related to the job description.
    JobRelated,
    /// Lorem ipsum padding.
    Lorem,
}

/// Source of the job advertisement text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobAdSource {
    /// Load from a file.
    File,
    /// Provided inline.
    Inline,
    /// Load from a cache ID.
    CacheId,
}

/// Placement of the job ad injection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobAdPlacement {
    /// Place at the front of the document.
    Front,
    /// Place at the back of the document.
    Back,
    /// Place after the summary section.
    AfterSummary,
    /// Custom placement.
    Custom,
}

/// Content configuration for the injection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InjectionContent {
    /// List of phrases to inject.
    #[serde(default)]
    pub phrases: Vec<String>,
    /// How the content is generated.
    #[serde(default)]
    pub generation_type: GenerationType,
    /// Job description for ad-targeted pollution.
    #[serde(default)]
    pub job_description: Option<String>,
}

impl Default for InjectionContent {
    fn default() -> Self {
        Self {
            phrases: vec![],
            generation_type: GenerationType::Static,
            job_description: None,
        }
    }
}

/// Configuration for the injection profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfileConfig {
    /// Visible block of meta-instructions.
    VisibleMetaBlock {
        /// Position of the block.
        position: InjectionPosition,
        /// Intensity of the instructions.
        intensity: Intensity,
        /// Content configuration.
        #[serde(default)]
        content: InjectionContent,
    },
    /// Low-visibility block (small font, low contrast).
    LowVisibilityBlock {
        /// Minimum font size.
        font_size_min: u8,
        /// Maximum font size.
        font_size_max: u8,
        /// Color palette to use.
        color_profile: LowVisibilityPalette,
        /// Content configuration.
        #[serde(default)]
        content: InjectionContent,
    },
    /// Text placed off the visible page area.
    OffpageLayer {
        /// Offset strategy.
        offset_strategy: OffpageOffset,
        /// Content configuration.
        #[serde(default)]
        content: InjectionContent,
    },
    /// Tracking pixel injection (URL link).
    TrackingPixel {
        /// The URL to track.
        url: String,
    },
    /// Code injection (JavaScript).
    CodeInjection {
        /// The JavaScript payload.
        payload: String,
    },
    /// Text hidden behind other content.
    UnderlayText,
    /// Injection into structural fields (metadata, tags).
    StructuralFields {
        /// Targets for injection.
        targets: Vec<StructuralTarget>,
    },
    /// Noise padding around content.
    PaddingNoise {
        /// Number of tokens before.
        padding_tokens_before: usize,
        /// Number of tokens after.
        padding_tokens_after: usize,
        /// Style of padding.
        padding_style: PaddingStyle,
        /// Content configuration.
        #[serde(default)]
        content: InjectionContent,
    },
    /// Inline job advertisement injection.
    InlineJobAd {
        /// Source of the job ad.
        job_ad_source: JobAdSource,
        /// Placement of the ad.
        placement: JobAdPlacement,
        /// Ratio of the ad to include.
        ad_excerpt_ratio: f32,
        /// Content configuration.
        #[serde(default)]
        content: InjectionContent,
    },
}

impl ProfileConfig {
    /// Returns the unique ID of the profile configuration type.
    pub fn id(&self) -> &'static str {
        match self {
            ProfileConfig::VisibleMetaBlock { .. } => "pdf.visible_meta_block",
            ProfileConfig::LowVisibilityBlock { .. } => "pdf.low_visibility_block",
            ProfileConfig::OffpageLayer { .. } => "pdf.offpage_layer",
            ProfileConfig::UnderlayText => "pdf.underlay_text",
            ProfileConfig::StructuralFields { .. } => "pdf.structural_fields",
            ProfileConfig::PaddingNoise { .. } => "pdf.padding_noise",
            ProfileConfig::InlineJobAd { .. } => "pdf.inline_job_ad",
            ProfileConfig::TrackingPixel { .. } => "pdf.tracking_pixel",
            ProfileConfig::CodeInjection { .. } => "pdf.code_injection",
        }
    }
}
