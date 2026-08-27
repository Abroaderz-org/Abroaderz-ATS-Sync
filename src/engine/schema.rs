use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateRecord {
    #[serde(default = "default_none")]
    pub name: String,
    #[serde(default = "default_none")]
    pub passport_no: String,
    #[serde(default = "default_none")]
    pub position: String,
    #[serde(default = "default_none")]
    pub education: String,
    #[serde(default = "default_none")]
    pub dob: String,
    #[serde(default = "default_none")]
    pub phone: String,
    #[serde(default = "default_none")]
    pub email: String,
    #[serde(default)]
    pub local_experience: f32,
    #[serde(default)]
    pub overseas_experience: f32,
    #[serde(default)]
    pub total_experience: f32,
    #[serde(default)]
    pub match_score: f32,
    #[serde(default = "default_none")]
    pub state: String,
    #[serde(default = "default_none")]
    pub country: String,
}

fn default_none() -> String {
    "None".to_string()
}

impl Default for CandidateRecord {
    fn default() -> Self {
        Self {
            name: default_none(),
            passport_no: default_none(),
            position: default_none(),
            education: default_none(),
            dob: default_none(),
            phone: default_none(),
            email: default_none(),
            local_experience: 0.0,
            overseas_experience: 0.0,
            total_experience: 0.0,
            match_score: 0.0,
            state: default_none(),
            country: default_none(),
        }
    }
}