use crate::engine::schema::CandidateRecord;
use std::error::Error;

pub fn export_candidates_to_csv(
    candidates: &[CandidateRecord],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(output_path)?;

    wtr.write_record([
        "Sno", "Name", "Passport No", "Position", "Education", "DOB",
        "Phone", "Email", "Local Exp", "Overseas Exp", "Total Exp",
        "State", "Country",
    ])?;

    for (idx, c) in candidates.iter().enumerate() {
        wtr.write_record(&[
            (idx + 1).to_string(),
            c.name.clone(),
            c.passport_no.clone(),
            c.position.clone(),
            c.education.clone(),
            c.dob.clone(),
            c.phone.clone(),
            c.email.clone(),
            c.local_experience.clone(),
            c.overseas_experience.clone(),
            c.total_experience.clone(),
            c.state.clone(),
            c.country.clone(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}