use crate::engine::schema::CandidateRecord;
use std::error::Error;

pub fn export_candidates_to_csv(
    candidates: &[CandidateRecord],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(output_path)?;

    let has_score = candidates.first().map_or(false, |c| c.match_score.is_some());

    let mut headers = vec![
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
        "State",
        "Country",
    ];

    if has_score {
        headers.push("Match Score (%)");
    }

    wtr.write_record(&headers)?;

    for (idx, c) in candidates.iter().enumerate() {
        let mut row = vec![
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
            c.state.clone(),
            c.country.clone(),
        ];

        if let Some(score) = c.match_score {
            row.push(format!("{:.1}", score));
        }

        wtr.write_record(&row)?;
    }

    wtr.flush()?;
    Ok(())
}