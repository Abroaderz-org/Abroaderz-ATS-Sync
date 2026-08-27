use crate::engine::schema::CandidateRecord;
use std::error::Error;

pub fn export_candidates_to_csv(
    candidates: &[CandidateRecord],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(output_path)?;

    wtr.write_record(&[
        "S.No",
        "Candidate Name",
        "Passport No",
        "Position / Trade",
        "Education / Degree",
        "Date of Birth",
        "Phone Number",
        "Email Address",
        "Local Exp (Yrs)",
        "Overseas Exp (Yrs)",
        "Total Exp (Yrs)",
        "Match Score (%)",
        "State",
        "Country",
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
            format!("{:.1}", c.local_experience),
            format!("{:.1}", c.overseas_experience),
            format!("{:.1}", c.total_experience),
            format!("{:.1}", c.match_score),
            c.state.clone(),
            c.country.clone(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}