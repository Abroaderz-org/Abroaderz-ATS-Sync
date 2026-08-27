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
    #[serde(default = "default_none")]
    pub local_experience: String,
    #[serde(default = "default_none")]
    pub overseas_experience: String,
    #[serde(default = "default_none")]
    pub total_experience: String,
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
            local_experience: default_none(),
            overseas_experience: default_none(),
            total_experience: default_none(),
            state: default_none(),
            country: default_none(),
        }
    }
}