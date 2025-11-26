use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatexResume {
    pub personal_info: PersonalInfo,
    pub sections: Vec<ResumeSection>,
    pub template: LatexTemplate,
    pub font: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersonalInfo {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub linkedin: String,
    pub github: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeSection {
    pub id: String, // For DnD
    pub title: String,
    pub items: Vec<SectionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionItem {
    pub id: String, // For DnD
    pub title: String,
    pub subtitle: String,
    pub date: String,
    pub description: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LatexTemplate {
    Modern,
    Classic,
    Minimal,
}

impl Default for LatexTemplate {
    fn default() -> Self {
        LatexTemplate::Modern
    }
}

use crate::generator::ScrapedProfile;

impl LatexResume {
    pub fn import_from_profile(&mut self, profile: &ScrapedProfile) {
        self.personal_info.name = profile.name.clone();
        self.personal_info.email = "".to_string(); // Not in scraped data
        self.personal_info.phone = "".to_string(); // Not in scraped data
        self.personal_info.linkedin = profile.url.clone();
        self.personal_info.github = "".to_string(); // Not in scraped data

        self.sections.clear();

        // Experience
        if !profile.experience.is_empty() {
            let mut items = Vec::new();
            for exp in &profile.experience {
                items.push(SectionItem {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: exp.title.clone(),
                    subtitle: exp.company.clone(),
                    date: exp.date_range.clone(),
                    description: vec![format!("Location: {}", exp.location)], // Use location as description
                });
            }
            self.sections.push(ResumeSection {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Experience".to_string(),
                items,
            });
        }

        // Education
        if !profile.education.is_empty() {
            let mut items = Vec::new();
            for edu in &profile.education {
                items.push(SectionItem {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: edu.degree.clone(),
                    subtitle: edu.school.clone(),
                    date: "".to_string(), // Not in scraped data
                    description: vec![],
                });
            }
            self.sections.push(ResumeSection {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Education".to_string(),
                items,
            });
        }

        // Skills
        if !profile.skills.is_empty() {
            self.sections.push(ResumeSection {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Skills".to_string(),
                items: vec![SectionItem {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: "Technical Skills".to_string(),
                    subtitle: "".to_string(),
                    date: "".to_string(),
                    description: profile.skills.clone(),
                }],
            });
        }
    }

    pub fn generate_latex(&self) -> String {
        let mut latex = String::new();
        
        // Header
        latex.push_str(r"\documentclass[11pt,a4paper]{article}
\usepackage[utf8]{inputenc}
\usepackage{geometry}
\geometry{left=2cm,right=2cm,top=2cm,bottom=2cm}
\usepackage{hyperref}
\usepackage{enumitem}
");
        
        if !self.font.is_empty() && self.font != "Default" {
             latex.push_str(&format!(r"\usepackage{{{}}}
", self.font.to_lowercase().replace(" ", "")));
        }

        latex.push_str(r"
\begin{document}

");

        // Personal Info
        latex.push_str(&format!(r"\begin{{center}}
    {{\LARGE \textbf{{{}}}}} \\ \vspace{{5pt}}
    {} | {} | {} | {}
\end{{center}}
\vspace{{10pt}}
", 
            self.personal_info.name,
            self.personal_info.email,
            self.personal_info.phone,
            self.personal_info.linkedin,
            self.personal_info.github
        ));

        // Sections
        for section in &self.sections {
            latex.push_str(&format!(r"\section*{{{}}}
\hrule
\vspace{{5pt}}
", section.title.to_uppercase()));

            for item in &section.items {
                latex.push_str(&format!(r"\noindent \textbf{{{}}} \hfill {} \\
\textit{{{}}}
\begin{{itemize}}[noitemsep,topsep=0pt]
", item.title, item.date, item.subtitle));

                for desc in &item.description {
                    latex.push_str(&format!(r"    \item {}
", desc));
                }
                latex.push_str(r"\end{itemize}
\vspace{5pt}
");
            }
        }

        latex.push_str(r"\end{document}");
        latex
    }
}
