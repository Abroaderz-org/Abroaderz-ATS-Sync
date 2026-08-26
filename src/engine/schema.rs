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
