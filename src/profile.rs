use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactInfo {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub websites: Vec<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Experience {
    pub title: String,
    pub company: String,
    pub location: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub summary: Option<String>,
    pub bullets: Vec<String>,
    pub tech_stack: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Education {
    pub institution: String,
    pub degree: Option<String>,
    pub field_of_study: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Skill {
    pub name: String,
    pub category: Option<String>,
    pub proficiency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub technologies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Certification {
    pub name: String,
    pub issuer: Option<String>,
    pub date: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Publication {
    pub title: String,
    pub publisher: Option<String>,
    pub date: Option<String>,
    pub link: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Volunteering {
    pub organization: String,
    pub role: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Language {
    pub name: String,
    pub proficiency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileMeta {
    pub audit_tags: Vec<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Seniority {
    Junior,
    Mid,
    Senior,
    Lead,
    Principal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiAtsMetadata {
    pub role_targets: Vec<String>,
    pub seniority: Option<Seniority>,
    pub domains: Vec<String>,
    pub skills_taxonomy: Vec<String>,
    pub keywords: Vec<String>,
    pub notes_for_human_reviewer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub headline: Option<String>,
    pub location: Option<String>,
    pub summary: Option<String>,
    pub contact: ContactInfo,
    pub experience: Vec<Experience>,
    pub education: Vec<Education>,
    pub skills: Vec<Skill>,
    pub projects: Vec<Project>,
    pub certifications: Vec<Certification>,
    pub publications: Vec<Publication>,
    pub volunteering: Vec<Volunteering>,
    pub languages: Vec<Language>,
    pub meta: Option<ProfileMeta>,
    pub ai_metadata: Option<AiAtsMetadata>,
}
