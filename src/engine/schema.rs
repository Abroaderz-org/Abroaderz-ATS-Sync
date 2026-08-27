use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateRecord {
    #[serde(default = "default_not_found")]
    pub name: String,
    #[serde(default = "default_not_found")]
    pub passport_no: String,
    #[serde(default = "default_not_found")]
    pub position: String,
    #[serde(default = "default_not_found")]
    pub education: String,
    #[serde(default = "default_not_found")]
    pub dob: String,
    #[serde(default = "default_not_found")]
    pub phone: String,
    #[serde(default = "default_not_found")]
    pub email: String,
    #[serde(default = "default_not_found")]
    pub local_experience: String,
    #[serde(default = "default_not_found")]
    pub overseas_experience: String,
    #[serde(default = "default_not_found")]
    pub total_experience: String,
    #[serde(default = "default_not_found")]
    pub state: String,
    #[serde(default = "default_not_found")]
    pub country: String,
    #[serde(default)]
    pub score: Option<String>,
}

fn default_not_found() -> String {
    "Not Found".to_string()
}

impl Default for CandidateRecord {
    fn default() -> Self {
        Self {
            name: default_not_found(),
            passport_no: default_not_found(),
            position: default_not_found(),
            education: default_not_found(),
            dob: default_not_found(),
            phone: default_not_found(),
            email: default_not_found(),
            local_experience: default_not_found(),
            overseas_experience: default_not_found(),
            total_experience: default_not_found(),
            state: default_not_found(),
            country: default_not_found(),
            score: None,
        }
    }
}