use crate::engine::schema::CandidateRecord;
use rust_xlsxwriter::{Format, FormatBorder, Workbook, XlsxError};

pub fn export_candidates_to_excel(
    candidates: &[CandidateRecord],
    output_path: &str,
) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let header_format = Format::new()
        .set_bold()
        .set_background_color("#1E3A8A")
        .set_font_color("#FFFFFF")
        .set_border(FormatBorder::Thin);

    let headers = [
        "Sno", "Name", "Passport No", "Position", "Education", "DOB",
        "Phone", "Email", "Local Exp", "Overseas Exp", "Total Exp",
        "State", "Country", "Score",
    ];

    for (col_idx, header) in headers.iter().enumerate() {
        worksheet.write_with_format(0, col_idx as u16, *header, &header_format)?;
    }

    for (row_idx, c) in candidates.iter().enumerate() {
        let row = (row_idx + 1) as u32;
        worksheet.write(row, 0, (row_idx + 1) as u32)?;
        worksheet.write(row, 1, &c.name)?;
        worksheet.write(row, 2, &c.passport_no)?;
        worksheet.write(row, 3, &c.position)?;
        worksheet.write(row, 4, &c.education)?;
        worksheet.write(row, 5, &c.dob)?;
        worksheet.write(row, 6, &c.phone)?;
        worksheet.write(row, 7, &c.email)?;
        worksheet.write(row, 8, &c.local_experience)?;
        worksheet.write(row, 9, &c.overseas_experience)?;
        worksheet.write(row, 10, &c.total_experience)?;
        worksheet.write(row, 11, &c.state)?;
        worksheet.write(row, 12, &c.country)?;
        worksheet.write(row, 13, c.score.as_deref().unwrap_or("N/A"))?;
    }

    workbook.save(output_path)?;
    Ok(())
}
