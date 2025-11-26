use crate::llm::LlmClient;
use crate::config::AppConfig;
use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AtsSimulationResult {
    pub candidate_name: Option<String>,
    pub email: Option<String>,
    pub skills_identified: Vec<String>,
    pub experience_timeline: Vec<AtsExperience>,
    pub missing_entities: Vec<String>,
    pub parsing_score: u8, // 0-100
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AtsExperience {
    pub role: String,
    pub company: String,
    pub duration: String,
}

pub struct AtsSimulator {
    llm_client: LlmClient,
}

impl AtsSimulator {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            llm_client: LlmClient::new(config.llm.clone()),
        }
    }

    pub fn simulate_parsing(&self, pdf_text: &str) -> Result<AtsSimulationResult> {
        let prompt = format!(
            r#"You are an Applicant Tracking System (ATS) simulator. 
            Analyze the following raw text extracted from a PDF resume. 
            Extract the structured data as if you were a machine parser.
            Identify any missing critical entities (Name, Email, Phone).
            Rate the parsing success from 0 to 100 based on how easily the data was extracted.

            Return ONLY a JSON object with the following structure:
            {{
                "candidate_name": "...",
                "email": "...",
                "skills_identified": ["skill1", "skill2"],
                "experience_timeline": [
                    {{ "role": "...", "company": "...", "duration": "..." }}
                ],
                "missing_entities": ["Phone", "Address"],
                "parsing_score": 85
            }}

            Raw Text:
            {}
            "#,
            pdf_text
        );

        let response = self.llm_client.generate(&prompt).map_err(|e| crate::AnalysisError::LlmError(e.to_string()))?;
        
        // Clean up response if it contains markdown code blocks
        let json_str = response.trim();
        let json_str = if json_str.starts_with("```json") {
            json_str.strip_prefix("```json").unwrap().strip_suffix("```").unwrap_or(json_str)
        } else if json_str.starts_with("```") {
             json_str.strip_prefix("```").unwrap().strip_suffix("```").unwrap_or(json_str)
        } else {
            json_str
        };

        let result: AtsSimulationResult = serde_json::from_str(json_str)
            .map_err(|e| crate::AnalysisError::JsonError(format!("Failed to parse ATS simulation JSON: {}. Response: {}", e, response)))?;

        Ok(result)
    }
}
