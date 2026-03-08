use lopdf::{Document, Object, dictionary, content::{Content, Operation}};
use crate::Result;
use crate::AnalysisError;

/// Adds text to a specific page at given coordinates.
pub fn add_text_to_page(
    doc: &mut Document,
    page_number: u32,
    text: &str,
    x: f64,
    y: f64,
    font_size: f64,
    color_gray: f64,
) -> Result<()> {
    let pages = doc.get_pages();
    let page_id = *pages.get(&page_number).ok_or_else(|| AnalysisError::PdfError(format!("Page {} not found", page_number)))?;

    // Ensure font exists
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    // Add font to resources
    let page = doc.get_object(page_id).unwrap().as_dict().unwrap();
    let resources_id = match page.get(b"Resources") {
        Ok(Object::Reference(id)) => *id,
        _ => {
            // If missing or direct, create a new indirect object for resources
            // Note: If it was direct, we lose it here (simplified for now)
            let res_id = doc.add_object(dictionary! {});
            let page_mut = doc.get_object_mut(page_id).unwrap().as_dict_mut().unwrap();
            page_mut.set("Resources", Object::Reference(res_id));
            res_id
        }
    };

    if let Ok(resources) = doc.get_object_mut(resources_id) {
        if let Object::Dictionary(dict) = resources {
            if !dict.has(b"Font") {
                dict.set("Font", dictionary! {});
            }
            let fonts = dict.get_mut(b"Font").unwrap().as_dict_mut().unwrap();
            fonts.set("F1", Object::Reference(font_id));
        }
    }

    // Create content stream
    let mut operations = Vec::new();
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new("Tf", vec!["F1".into(), font_size.into()]));
    operations.push(Operation::new("g", vec![color_gray.into()]));
    operations.push(Operation::new("Td", vec![x.into(), y.into()]));
    operations.push(Operation::new("Tj", vec![Object::string_literal(text)]));
    operations.push(Operation::new("ET", vec![]));

    let content = Content { operations };
    let content_stream = doc.add_object(lopdf::Stream::new(dictionary! {}, content.encode().unwrap()));

    // Append to page contents
    let page = doc.get_object_mut(page_id).unwrap().as_dict_mut().unwrap();
    
    // Determine the action based on current "Contents"
    enum Action {
        ReplaceWithArray(Vec<Object>),
        AppendToArray,
        SetNew(Object),
    }

    let action = if let Ok(contents) = page.get(b"Contents") {
        match contents {
            Object::Reference(id) => Action::ReplaceWithArray(vec![Object::Reference(*id), Object::Reference(content_stream)]),
            Object::Array(_) => Action::AppendToArray,
            _ => Action::SetNew(Object::Reference(content_stream)),
        }
    } else {
        Action::SetNew(Object::Reference(content_stream))
    };

    match action {
        Action::ReplaceWithArray(arr) => {
            page.set("Contents", arr);
        }
        Action::AppendToArray => {
            if let Ok(Object::Array(arr)) = page.get_mut(b"Contents") {
                arr.push(Object::Reference(content_stream));
            }
        }
        Action::SetNew(obj) => {
            page.set("Contents", obj);
        }
    }

    Ok(())
}

/// Adds text to a specific page at given coordinates, ensuring it is rendered *before* existing content (underlay).
pub fn prepend_text_to_page(
    doc: &mut Document,
    page_number: u32,
    text: &str,
    x: f64,
    y: f64,
    font_size: f64,
    color_gray: f64,
) -> Result<()> {
    let pages = doc.get_pages();
    let page_id = *pages.get(&page_number).ok_or_else(|| AnalysisError::PdfError(format!("Page {} not found", page_number)))?;

    // Ensure font exists (reuse logic or refactor if needed, for now duplicating for safety)
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    // Add font to resources
    let page = doc.get_object(page_id).unwrap().as_dict().unwrap();
    let resources_id = match page.get(b"Resources") {
        Ok(Object::Reference(id)) => *id,
        _ => {
            let res_id = doc.add_object(dictionary! {});
            let page_mut = doc.get_object_mut(page_id).unwrap().as_dict_mut().unwrap();
            page_mut.set("Resources", Object::Reference(res_id));
            res_id
        }
    };

    if let Ok(resources) = doc.get_object_mut(resources_id) {
        if let Object::Dictionary(dict) = resources {
            if !dict.has(b"Font") {
                dict.set("Font", dictionary! {});
            }
            let fonts = dict.get_mut(b"Font").unwrap().as_dict_mut().unwrap();
            fonts.set("F1", Object::Reference(font_id));
        }
    }

    // Create content stream
    let mut operations = Vec::new();
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new("Tf", vec!["F1".into(), font_size.into()]));
    operations.push(Operation::new("g", vec![color_gray.into()]));
    operations.push(Operation::new("Td", vec![x.into(), y.into()]));
    operations.push(Operation::new("Tj", vec![Object::string_literal(text)]));
    operations.push(Operation::new("ET", vec![]));

    let content = Content { operations };
    let content_stream = doc.add_object(lopdf::Stream::new(dictionary! {}, content.encode().unwrap()));

    // Prepend to page contents
    let page = doc.get_object_mut(page_id).unwrap().as_dict_mut().unwrap();
    
    enum Action {
        ReplaceWithArray(Vec<Object>),
        PrependToArray,
        SetNew(Object),
    }

    let action = if let Ok(contents) = page.get(b"Contents") {
        match contents {
            Object::Reference(id) => Action::ReplaceWithArray(vec![Object::Reference(content_stream), Object::Reference(*id)]),
            Object::Array(_) => Action::PrependToArray,
            _ => Action::SetNew(Object::Reference(content_stream)),
        }
    } else {
        Action::SetNew(Object::Reference(content_stream))
    };

    match action {
        Action::ReplaceWithArray(arr) => {
            page.set("Contents", arr);
        }
        Action::PrependToArray => {
            if let Ok(Object::Array(arr)) = page.get_mut(b"Contents") {
                arr.insert(0, Object::Reference(content_stream));
            }
        }
        Action::SetNew(obj) => {
            page.set("Contents", obj);
        }
    }

    Ok(())
}

/// Creates a blank PDF document.
pub fn create_blank_pdf() -> Document {
    let mut doc = Document::with_version("1.4");
    let pages_id = doc.new_object_id();
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    });
    let pages = dictionary! {
        "Type" => "Pages",
        "Kids" => vec![page_id.into()],
        "Count" => 1,
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages));
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    doc
}

/// Adds a Link Annotation to a page.
pub fn add_link_annotation(
    doc: &mut Document,
    page_number: u32,
    url: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<()> {
    let pages = doc.get_pages();
    let page_id = *pages.get(&page_number).ok_or_else(|| AnalysisError::PdfError(format!("Page {} not found", page_number)))?;

    let rect = vec![x.into(), y.into(), (x + width).into(), (y + height).into()];
    
    let action = dictionary! {
        "Type" => "Action",
        "S" => "URI",
        "URI" => Object::string_literal(url),
    };

    let annotation = dictionary! {
        "Type" => "Annot",
        "Subtype" => "Link",
        "Rect" => Object::Array(rect),
        "Border" => Object::Array(vec![0.into(), 0.into(), 0.into()]), // Invisible border
        "A" => action,
    };

    let annot_id = doc.add_object(annotation);

    let page = doc.get_object_mut(page_id).unwrap().as_dict_mut().unwrap();
    if !page.has(b"Annots") {
        page.set("Annots", Object::Array(vec![]));
    }

    if let Ok(Object::Array(annots)) = page.get_mut(b"Annots") {
        annots.push(Object::Reference(annot_id));
    }

    Ok(())
}

/// Adds a JavaScript action to the PDF's OpenAction.
pub fn add_javascript_action(doc: &mut Document, js: &str) -> Result<()> {
    let js_action = doc.add_object(dictionary! {
        "S" => "JavaScript",
        "JS" => Object::String(js.into(), lopdf::StringFormat::Literal),
    });

    doc.trailer.set("OpenAction", js_action);
    Ok(())
}

/// Extracts text from a PDF file (simplified).
pub fn extract_text_from_pdf(path: &std::path::Path) -> Result<String> {
    let doc = Document::load(path).map_err(|e| AnalysisError::PdfError(e.to_string()))?;
    let mut text = String::new();

    for page_id in doc.page_iter() {
        let content = doc.get_page_content(page_id).map_err(|e| AnalysisError::PdfError(e.to_string()))?;
        let content = Content::decode(&content).map_err(|e| AnalysisError::PdfError(e.to_string()))?;
        
        for operation in content.operations {
            match operation.operator.as_str() {
                "Tj" | "TJ" => {
                    // Extract text from Tj (show text) and TJ (show text with spacing)
                    for operand in operation.operands {
                        match operand {
                            Object::String(bytes, _) => {
                                if let Ok(s) = std::str::from_utf8(&bytes) {
                                    text.push_str(s);
                                } else {
                                    // Try lossy
                                    text.push_str(&String::from_utf8_lossy(&bytes));
                                }
                            }
                            Object::Array(arr) => {
                                for item in arr {
                                    if let Object::String(bytes, _) = item {
                                        if let Ok(s) = std::str::from_utf8(&bytes) {
                                            text.push_str(s);
                                        } else {
                                            text.push_str(&String::from_utf8_lossy(&bytes));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    text.push(' '); // Add space between text blocks
                }
                "ET" => {
                    text.push('\n'); // End of text object
                }
                _ => {}
            }
        }
        text.push('\n'); // End of page
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_blank_pdf() {
        let doc = create_blank_pdf();
        let pages = doc.get_pages();
        assert_eq!(pages.len(), 1, "blank PDF should have one page");
    }

    #[test]
    fn test_add_text_to_page() {
        let mut doc = create_blank_pdf();
        let result = add_text_to_page(&mut doc, 1, "hello world", 50.0, 700.0, 12.0, 0.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_prepend_text_to_page() {
        let mut doc = create_blank_pdf();
        // First add some content so there is something to prepend before
        add_text_to_page(&mut doc, 1, "existing", 50.0, 700.0, 12.0, 0.0).unwrap();
        let result = prepend_text_to_page(&mut doc, 1, "prepended", 50.0, 400.0, 10.0, 1.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_text_from_pdf() {
        let mut doc = create_blank_pdf();
        add_text_to_page(&mut doc, 1, "SuperpoweredCV Test Content", 50.0, 700.0, 12.0, 0.0)
            .unwrap();

        let dir = std::env::temp_dir().join("superpoweredcv_pdfutils_tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("extract_test.pdf");
        doc.save(&path).unwrap();

        let text = extract_text_from_pdf(&path).expect("extract text");
        assert!(
            text.contains("SuperpoweredCV Test Content"),
            "Extracted text should contain the injected content, got: {}",
            text
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
