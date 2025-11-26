use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatexResume {
    pub personal_info: PersonalInfo,
    pub sections: Vec<ResumeSection>,
    pub template: LatexTemplate,
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
    pub title: String,
    pub items: Vec<SectionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionItem {
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

impl LatexResume {
    pub fn generate_latex(&self) -> String {
        let mut latex = String::new();
        
        // Header
        latex.push_str(r"\documentclass[11pt,a4paper]{article}
\usepackage[utf8]{inputenc}
\usepackage{geometry}
\geometry{left=2cm,right=2cm,top=2cm,bottom=2cm}
\usepackage{hyperref}
\usepackage{enumitem}

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
