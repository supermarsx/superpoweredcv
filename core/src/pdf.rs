use crate::attacks::{ProfileConfig, InjectionPosition, LowVisibilityPalette, OffpageOffset, InjectionContent};
use crate::attacks::templates::InjectionTemplate;
use crate::Result;
use crate::pdf_utils;
use lopdf::{Document, Object, StringFormat, dictionary};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Request to mutate a PDF with a specific analysis profile and template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMutationRequest {
    /// Path to the base PDF file.
    pub base_pdf: PathBuf,
    /// Configuration for the analysis profiles (multiple allowed).
    pub profiles: Vec<ProfileConfig>,
    /// The analysis template to use (for default text if needed).
    pub template: InjectionTemplate,
    /// Optional ID for the variant.
    pub variant_id: Option<String>,
}

/// Result of a PDF mutation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMutationResult {
    /// Unique ID of the generated variant.
    pub variant_id: String,
    /// Path to the mutated PDF file.
    pub mutated_pdf: PathBuf,
    /// Hash of the mutated PDF content.
    pub variant_hash: Option<String>,
    /// Notes or logs from the mutation process.
    pub notes: Vec<String>,
}

/// Trait for components that can mutate PDFs.
pub trait PdfMutator {
    /// Mutates a PDF based on the request.
    fn mutate(&self, request: PdfMutationRequest) -> Result<PdfMutationResult>;
}

/// A real PDF mutator that uses lopdf to modify PDF files.
pub struct RealPdfMutator {
    /// Directory where mutated PDFs will be saved.
    pub output_dir: PathBuf,
}

impl RealPdfMutator {
    /// Creates a new `RealPdfMutator` with the specified output directory.
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        RealPdfMutator {
            output_dir: output_dir.into(),
        }
    }
}

impl PdfMutator for RealPdfMutator {
    fn mutate(&self, request: PdfMutationRequest) -> Result<PdfMutationResult> {
        fs::create_dir_all(&self.output_dir)?;

        let variant_id = request
            .variant_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let file_name = format!("{}.pdf", variant_id);
        let output_path = self.output_dir.join(file_name);

        // Load the base PDF
        let mut doc = Document::load(&request.base_pdf)
            .map_err(|e| crate::AnalysisError::PdfError(format!("Failed to load PDF: {}", e)))?;

        let mut notes = Vec::new();
        let default_text = &request.template.text_template;
        let mut final_injected_text = default_text.clone();

        for profile in &request.profiles {
            match profile {
                ProfileConfig::VisibleMetaBlock { position, intensity: _, content } => {
                    let text_to_inject = get_injection_text(content, default_text);
                    final_injected_text = text_to_inject.clone();
                    let (x, y) = match position {
                        InjectionPosition::Header => (50.0, 800.0),
                        InjectionPosition::Footer => (50.0, 50.0),
                        InjectionPosition::Section(_) => (50.0, 400.0), // Default to middle for now
                    };
                    // Inject on the first page
                    pdf_utils::add_text_to_page(&mut doc, 1, &text_to_inject, x, y, 10.0, 0.0)?;
                    notes.push(format!("Injected visible block at {:?} ({}, {})", position, x, y));
                }
                ProfileConfig::LowVisibilityBlock { font_size_min, color_profile, content, .. } => {
                    let text_to_inject = get_injection_text(content, default_text);
                    final_injected_text = text_to_inject.clone();
                    let gray_level = match color_profile {
                        LowVisibilityPalette::Gray => 0.95,
                        LowVisibilityPalette::LightBlue => 0.90, // Simplified to gray for now
                        LowVisibilityPalette::OffWhite => 0.99,
                    };
                    // Inject at bottom
                    pdf_utils::add_text_to_page(&mut doc, 1, &text_to_inject, 50.0, 20.0, *font_size_min as f64, gray_level)?;
                    notes.push(format!("Injected low visibility block (size: {}, gray: {})", font_size_min, gray_level));
                }
                ProfileConfig::OffpageLayer { offset_strategy, content, .. } => {
                    let text_to_inject = get_injection_text(content, default_text);
                    final_injected_text = text_to_inject.clone();
                    let (x, y) = match offset_strategy {
                        OffpageOffset::BottomClip => (50.0, -1000.0),
                        OffpageOffset::RightClip => (1000.0, 500.0),
                    };
                    pdf_utils::add_text_to_page(&mut doc, 1, &text_to_inject, x, y, 1.0, 0.0)?;
                    notes.push(format!("Injected offpage layer at ({}, {})", x, y));
                }
                ProfileConfig::UnderlayText => {
                    // Inject text behind existing content (e.g. white text or just first in stream)
                    // We use a large font size to cover area, but white color so it's invisible to human eye
                    // but present in stream. Or we can use black text if we are sure it's covered by an image.
                    // For safety/simplicity, we use white text (invisible) but placed first.
                    // Actually, spec says "invisible but still selectable".
                    let text_to_inject = default_text.clone();
                    final_injected_text = text_to_inject.clone();
                    pdf_utils::prepend_text_to_page(&mut doc, 1, &text_to_inject, 50.0, 400.0, 12.0, 1.0)?; // 1.0 is white in Gray colorspace
                    notes.push("Injected underlay text (white, prepended to stream)".to_string());
                }
                ProfileConfig::StructuralFields { targets } => {
                    let text_to_inject = default_text.clone();
                    final_injected_text = text_to_inject.clone();
                    
                    let info_id = match doc.trailer.get(b"Info").ok().and_then(|obj| obj.as_reference().ok()) {
                        Some(id) => id,
                        None => {
                            let info_id = doc.add_object(dictionary! {});
                            doc.trailer.set("Info", info_id);
                            info_id
                        }
                    };

                    if let Ok(info) = doc.get_object_mut(info_id) {
                        if let Object::Dictionary(dict) = info {
                            for target in targets {
                                match target {
                                    crate::attacks::StructuralTarget::AltText => {
                                        // Simulating AltText by adding a custom key, as real AltText requires structure tree
                                        dict.set("AltTextInjection", Object::String(text_to_inject.clone().into(), StringFormat::Literal));
                                        notes.push("Injected into Info dict (simulated AltText)".to_string());
                                    }
                                    crate::attacks::StructuralTarget::PdfTag => {
                                        dict.set("Keywords", Object::String(text_to_inject.clone().into(), StringFormat::Literal));
                                        notes.push("Injected into Keywords".to_string());
                                    }
                                    crate::attacks::StructuralTarget::XmpMetadata => {
                                        dict.set("Subject", Object::String(text_to_inject.clone().into(), StringFormat::Literal));
                                        notes.push("Injected into Subject".to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                ProfileConfig::PaddingNoise { padding_tokens_before, padding_tokens_after, padding_style, content } => {
                    let noise_before = generate_noise(Some(*padding_tokens_before as u32), None, padding_style);
                    let noise_after = generate_noise(None, Some(*padding_tokens_after as u32), padding_style);
                    let text_to_inject = get_injection_text(content, default_text);
                    
                    let full_text = format!("{} {} {}", noise_before, text_to_inject, noise_after);
                    final_injected_text = full_text.clone();
                    
                    // Inject as low visibility text at the end
                    pdf_utils::add_text_to_page(&mut doc, 1, &full_text, 50.0, 10.0, 1.0, 0.99)?;
                    notes.push(format!("Injected padding noise ({:?}) with content", padding_style));
                }
                ProfileConfig::InlineJobAd { job_ad_source, placement, ad_excerpt_ratio: _, content } => {
                    let ad_text = match job_ad_source {
                        crate::attacks::JobAdSource::Inline => {
                            content.job_description.clone().unwrap_or_else(|| "Senior Software Engineer required. Must have Rust experience.".to_string())
                        }
                        _ => content.job_description.clone().unwrap_or_else(|| "Job Ad Content".to_string()),
                    };
                    let text_to_inject = get_injection_text(content, default_text);
                    let full_text = format!("{} {}", text_to_inject, ad_text);
                    final_injected_text = full_text.clone();
                    
                    let (x, y) = match placement {
                        crate::attacks::JobAdPlacement::Front => (50.0, 800.0),
                        crate::attacks::JobAdPlacement::Back => (50.0, 50.0),
                        _ => (50.0, 50.0),
                    };
                    
                    // Inject as visible text (or low vis depending on intent, assuming visible for now based on name)
                    // Spec says "Inline Job Ad", usually implies visible or hidden. Let's assume hidden/low-vis for red-teaming context usually,
                    // but "Inline" might mean visible. Let's use small white text for safety in this context.
                    pdf_utils::add_text_to_page(&mut doc, 1, &full_text, x, y, 4.0, 0.95)?;
                    notes.push(format!("Injected inline job ad ({:?}) with content", placement));
                }
                ProfileConfig::TrackingPixel { url } => {
                    // Inject a URI Action on a Link Annotation (invisible rectangle)
                    // This is the most reliable way to trigger a network request on click, 
                    // but for "open" tracking, we might try an external XObject or just a link covering the whole page.
                    // Here we add a link covering the top of the page.
                    pdf_utils::add_link_annotation(&mut doc, 1, url, 0.0, 0.0, 600.0, 850.0)?;
                    notes.push(format!("Injected tracking link (covering page) to {}", url));
                }
                ProfileConfig::CodeInjection { payload } => {
                    // Inject JavaScript Action into the OpenAction of the PDF
                    pdf_utils::add_javascript_action(&mut doc, payload)?;
                    notes.push("Injected JavaScript OpenAction".to_string());
                }
            }
        }
        
        // Always inject metadata as a backup/marker
        let info_id = match doc.trailer.get(b"Info").ok().and_then(|obj| obj.as_reference().ok()) {
            Some(id) => id,
            None => {
                let info_id = doc.add_object(dictionary! {});
                doc.trailer.set("Info", info_id);
                info_id
            }
        };

        if let Ok(info) = doc.get_object_mut(info_id) {
            if let Object::Dictionary(dict) = info {
                dict.set(
                    "CustomInjection", 
                    Object::String(final_injected_text.into(), StringFormat::Literal)
                );
                dict.set(
                    "Producer",
                    Object::String("SuperpoweredCV Analysis Tool".into(), StringFormat::Literal)
                );
            }
        }

        // Save the mutated PDF
        let mut file = fs::File::create(&output_path)?;
        doc.save_to(&mut file)
            .map_err(|e| crate::AnalysisError::PdfError(format!("Failed to save PDF: {}", e)))?;

        // Calculate hash of the new file
        let mut hasher = Sha256::new();
        let content = fs::read(&output_path)?;
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());

        Ok(PdfMutationResult {
            variant_id,
            mutated_pdf: output_path,
            variant_hash: Some(hash),
            notes,
        })
    }
}

/// A placeholder mutator that writes a small marker file containing mutation
/// metadata. This gives downstream code a tangible artifact (with hash) without
/// requiring a PDF stack during early development.
pub struct StubPdfMutator {
    /// Directory where mutated PDFs (stubs) will be saved.
    pub output_dir: PathBuf,
}

impl StubPdfMutator {
    /// Creates a new `StubPdfMutator` with the specified output directory.
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        StubPdfMutator {
            output_dir: output_dir.into(),
        }
    }
}

impl PdfMutator for StubPdfMutator {
    fn mutate(&self, request: PdfMutationRequest) -> Result<PdfMutationResult> {
        fs::create_dir_all(&self.output_dir)?;

        let variant_id = request
            .variant_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let file_name = format!("{}.pdf", variant_id);
        let output_path = self.output_dir.join(file_name);

        // In a real implementation, this would apply the injection.
        // Here we just copy the base PDF if it exists, or create a dummy one.
        if request.base_pdf.exists() {
            fs::copy(&request.base_pdf, &output_path)?;
        } else {
            // Create a dummy PDF file for testing
            fs::write(&output_path, b"%PDF-1.4\n%Dummy PDF content for testing")?;
        }

        // Calculate a simple hash of the output file
        let mut hasher = Sha256::new();
        let content = fs::read(&output_path)?;
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());

        Ok(PdfMutationResult {
            variant_id,
            mutated_pdf: output_path,
            variant_hash: Some(hash),
            notes: vec![
                "Stub mutator: copied base PDF (or created dummy)".into(),
                format!("Applied profile: {:?}", request.profiles),
            ],
        })
    }
}

pub(crate) fn get_injection_text(content: &InjectionContent, default: &str) -> String {
    if !content.phrases.is_empty() {
        content.phrases.join("\n")
    } else {
        default.to_string()
    }
}

pub(crate) fn generate_noise(before: Option<u32>, after: Option<u32>, style: &crate::attacks::PaddingStyle) -> String {
    let count_before = before.unwrap_or(0);
    let count_after = after.unwrap_or(0);
    let total = count_before + count_after;
    
    if total == 0 {
        return String::new();
    }

    match style {
        crate::attacks::PaddingStyle::Lorem => {
            let words = ["lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit"];
            (0..total).map(|i| words[(i as usize) % words.len()]).collect::<Vec<_>>().join(" ")
        }
        crate::attacks::PaddingStyle::ResumeLike => {
            let words = ["experience", "team", "led", "developed", "managed", "project", "skills", "communication"];
            (0..total).map(|i| words[(i as usize) % words.len()]).collect::<Vec<_>>().join(" ")
        }
        crate::attacks::PaddingStyle::JobRelated => {
            let words = ["requirements", "qualifications", "responsibilities", "role", "candidate", "apply"];
            (0..total).map(|i| words[(i as usize) % words.len()]).collect::<Vec<_>>().join(" ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attacks::{
        InjectionContent, InjectionPosition, Intensity, LowVisibilityPalette,
        OffpageOffset, PaddingStyle, ProfileConfig, StructuralTarget,
    };
    use crate::attacks::templates::{
        ControlType, GenerationType, InjectionTemplate, TemplateSeverity, TemplateStyle,
    };

    fn sample_template() -> InjectionTemplate {
        InjectionTemplate {
            id: "test_tmpl".into(),
            severity: TemplateSeverity::Low,
            goal: "test".into(),
            style: TemplateStyle::Subtle,
            control: ControlType::Plain,
            text_template: "Test injection text".into(),
            phrases: vec![],
            generation_type: GenerationType::Static,
            job_description: None,
        }
    }

    fn setup(name: &str) -> (PathBuf, PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("superpoweredcv_pdf_{}", name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let base_path = dir.join("base.pdf");
        let mut doc = crate::pdf_utils::create_blank_pdf();
        doc.save(&base_path).unwrap();
        let out_dir = dir.join("output");
        std::fs::create_dir_all(&out_dir).unwrap();
        (dir, base_path, out_dir)
    }

    #[test]
    fn test_stub_mutator_creates_output() {
        let (dir, base, out_dir) = setup("stub");
        let mutator = StubPdfMutator::new(&out_dir);
        let result = mutator
            .mutate(PdfMutationRequest {
                base_pdf: base,
                profiles: vec![ProfileConfig::UnderlayText],
                template: sample_template(),
                variant_id: Some("stub_test".into()),
            })
            .expect("stub mutate");
        assert!(result.mutated_pdf.exists());
        assert!(result.variant_hash.is_some());
        assert_eq!(result.variant_id, "stub_test");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_real_mutator_visible_meta_block() {
        let (dir, base, out_dir) = setup("vmb");
        let mutator = RealPdfMutator::new(&out_dir);
        let result = mutator
            .mutate(PdfMutationRequest {
                base_pdf: base,
                profiles: vec![ProfileConfig::VisibleMetaBlock {
                    position: InjectionPosition::Footer,
                    intensity: Intensity::Soft,
                    content: InjectionContent::default(),
                }],
                template: sample_template(),
                variant_id: Some("vmb_test".into()),
            })
            .expect("real mutate visible meta block");
        assert!(result.mutated_pdf.exists());
        assert!(result.notes.iter().any(|n| n.contains("visible block")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_real_mutator_low_visibility() {
        let (dir, base, out_dir) = setup("lv");
        let mutator = RealPdfMutator::new(&out_dir);
        let result = mutator
            .mutate(PdfMutationRequest {
                base_pdf: base,
                profiles: vec![ProfileConfig::LowVisibilityBlock {
                    font_size_min: 1,
                    font_size_max: 3,
                    color_profile: LowVisibilityPalette::OffWhite,
                    content: InjectionContent::default(),
                }],
                template: sample_template(),
                variant_id: Some("lv_test".into()),
            })
            .expect("real mutate low visibility");
        assert!(result.mutated_pdf.exists());
        assert!(result.notes.iter().any(|n| n.contains("low visibility")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_real_mutator_offpage_layer() {
        let (dir, base, out_dir) = setup("opl");
        let mutator = RealPdfMutator::new(&out_dir);
        let result = mutator
            .mutate(PdfMutationRequest {
                base_pdf: base,
                profiles: vec![ProfileConfig::OffpageLayer {
                    offset_strategy: OffpageOffset::RightClip,
                    content: InjectionContent::default(),
                }],
                template: sample_template(),
                variant_id: Some("opl_test".into()),
            })
            .expect("real mutate offpage layer");
        assert!(result.mutated_pdf.exists());
        assert!(result.notes.iter().any(|n| n.contains("offpage")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_real_mutator_structural_fields() {
        let (dir, base, out_dir) = setup("sf");
        let mutator = RealPdfMutator::new(&out_dir);
        let result = mutator
            .mutate(PdfMutationRequest {
                base_pdf: base,
                profiles: vec![ProfileConfig::StructuralFields {
                    targets: vec![StructuralTarget::AltText, StructuralTarget::PdfTag],
                }],
                template: sample_template(),
                variant_id: Some("sf_test".into()),
            })
            .expect("real mutate structural fields");
        assert!(result.mutated_pdf.exists());
        assert!(result.notes.iter().any(|n| n.contains("Injected")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_real_mutator_tracking_pixel() {
        let (dir, base, out_dir) = setup("tp");
        let mutator = RealPdfMutator::new(&out_dir);
        let result = mutator
            .mutate(PdfMutationRequest {
                base_pdf: base,
                profiles: vec![ProfileConfig::TrackingPixel {
                    url: "https://example.com/pixel".into(),
                }],
                template: sample_template(),
                variant_id: Some("tp_test".into()),
            })
            .expect("real mutate tracking pixel");
        assert!(result.mutated_pdf.exists());
        assert!(result.notes.iter().any(|n| n.contains("tracking link")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_real_mutator_code_injection() {
        let (dir, base, out_dir) = setup("ci");
        let mutator = RealPdfMutator::new(&out_dir);
        let result = mutator
            .mutate(PdfMutationRequest {
                base_pdf: base,
                profiles: vec![ProfileConfig::CodeInjection {
                    payload: "app.alert('hello')".into(),
                }],
                template: sample_template(),
                variant_id: Some("ci_test".into()),
            })
            .expect("real mutate code injection");
        assert!(result.mutated_pdf.exists());
        assert!(result.notes.iter().any(|n| n.contains("JavaScript")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_get_injection_text_with_phrases() {
        let content = InjectionContent {
            phrases: vec!["phrase one".into(), "phrase two".into()],
            generation_type: GenerationType::Static,
            job_description: None,
        };
        let text = get_injection_text(&content, "default");
        assert_eq!(text, "phrase one\nphrase two");
    }

    #[test]
    fn test_get_injection_text_without_phrases() {
        let content = InjectionContent::default();
        let text = get_injection_text(&content, "fallback text");
        assert_eq!(text, "fallback text");
    }

    #[test]
    fn test_generate_noise() {
        let noise = generate_noise(Some(4), Some(4), &PaddingStyle::Lorem);
        assert!(!noise.is_empty());
        assert!(noise.contains("lorem"));

        let noise2 = generate_noise(Some(3), None, &PaddingStyle::ResumeLike);
        assert!(noise2.contains("experience"));

        let empty = generate_noise(None, None, &PaddingStyle::Lorem);
        assert!(empty.is_empty());
    }
}
