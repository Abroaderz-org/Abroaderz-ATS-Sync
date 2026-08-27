use crate::engine::schema::CandidateRecord;
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook, XlsxError};

pub fn export_candidates_to_excel(
    candidates: &[CandidateRecord],
    output_path: &str,
) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let header_format = Format::new()
        .set_bold()
        .set_background_color(rust_xlsxwriter::Color::RGB(0x0E, 0xA5, 0xE9))
        .set_font_color(rust_xlsxwriter::Color::RGB(0xFF, 0xFF, 0xFF))
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    let cell_format = Format::new()
        .set_align(FormatAlign::Left)
        .set_border(FormatBorder::Thin);

    let headers = [
        "S.No",
        "Candidate Name",
        "Passport No",
        "Position / Trade",
        "Education / Degree",
        "Date of Birth",
        "Phone Number",
        "Email Address",
        "Local Experience",
        "Overseas Experience",
        "Total Experience",
        "State",
        "Country",
    ];

    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, col as u16, *header, &header_format)?;
    }

    for (row_idx, c) in candidates.iter().enumerate() {
        let row = (row_idx + 1) as u32;
        worksheet.write_number_with_format(row, 0, (row_idx + 1) as f64, &cell_format)?;
        worksheet.write_string_with_format(row, 1, &c.name, &cell_format)?;
        worksheet.write_string_with_format(row, 2, &c.passport_no, &cell_format)?;
        worksheet.write_string_with_format(row, 3, &c.position, &cell_format)?;
        worksheet.write_string_with_format(row, 4, &c.education, &cell_format)?;
        worksheet.write_string_with_format(row, 5, &c.dob, &cell_format)?;
        worksheet.write_string_with_format(row, 6, &c.phone, &cell_format)?;
        worksheet.write_string_with_format(row, 7, &c.email, &cell_format)?;
        worksheet.write_string_with_format(row, 8, &c.local_experience, &cell_format)?;
        worksheet.write_string_with_format(row, 9, &c.overseas_experience, &cell_format)?;
        worksheet.write_string_with_format(row, 10, &c.total_experience, &cell_format)?;
        worksheet.write_string_with_format(row, 11, &c.state, &cell_format)?;
        worksheet.write_string_with_format(row, 12, &c.country, &cell_format)?;
    }

    worksheet.autofit();
    workbook.save(output_path)?;

    Ok(())
}