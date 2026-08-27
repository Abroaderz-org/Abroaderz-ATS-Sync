# Abroaderz ATS Sync ⚡

An automated resume parsing and candidate synchronization desktop application written in Rust. **Abroaderz ATS Sync** extracts key candidate data from resumes (PDF, DOCX, and images) and formats the output into clean, structured Excel worksheets and CSV reports.

---

## ✨ Features

- **Multi-Format Ingestion**: Supports `.pdf`, `.docx`, `.png`, `.jpg`, `.jpeg`, and `.webp` resumes.
- **Smart Data Extraction**:
  - **Candidate Identity**: Auto-cleans and infers names directly from file naming structures and document headers.
  - **Designation & Trade**: Matches engineering and vocational roles into normalized Title Case.
  - **Contact Details**: Flexible regex parsing for international phone numbers and email addresses.
  - **Passport & Experience**: Detects standard passport formats, total experience years, and GCC/overseas work history.
  - **Education**: Captures degree titles while filtering out excessive institution metadata.
- **Flexible Export**: Generate formatted **Excel (`.xlsx`)** workbooks with auto-fitted columns, lightweight **CSV (`.csv`)** files, or both simultaneously.
- **Native GUI**: Built using `egui` and `eframe` with dark/light mode support and direct report launcher shortcuts.

---

## 🚀 How to Use

### 1. Select Resume Folder
1. Launch `abroaderz-ats-sync.exe`.
2. Click **Browse** under **Source Directory** and choose the directory containing your candidate resumes.

### 2. Choose Export Format
Select your preferred export option:
- **Excel (.xlsx)**: Produces a styled spreadsheet with autofitted headers.
- **CSV (.csv)**: Generates a lightweight comma-separated file.
- **Both**: Creates both files in the project working directory.

### 3. Run Pipeline & Open Results
1. Click **⚡ Run ATS Extraction**.
2. Once the extraction status indicates success, click **📊 Open Excel** or **📄 Open CSV** to review the synchronized candidate records immediately.

---

## 🛠️ Building from Source

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (Stable toolchain)

### Steps

```powershell
# Clone the repository
git clone [https://github.com/Abroaderz-org/Abroaderz-ATS-Sync.git](https://github.com/Abroaderz-org/Abroaderz-ATS-Sync.git)
cd Abroaderz-ATS-Sync

# Run in debug mode
cargo run

# Build optimized release executable
cargo build --release
