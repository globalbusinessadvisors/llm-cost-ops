// Export formats (CSV, JSON, Excel)

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::io::Write;

use super::{ExportError, ExportResult};

/// Supported export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Csv,
    Json,
    Excel,
    JsonLines,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Csv => write!(f, "csv"),
            ExportFormat::Json => write!(f, "json"),
            ExportFormat::Excel => write!(f, "xlsx"),
            ExportFormat::JsonLines => write!(f, "jsonl"),
        }
    }
}

impl ExportFormat {
    pub fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "text/csv",
            ExportFormat::Json => "application/json",
            ExportFormat::Excel => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ExportFormat::JsonLines => "application/x-ndjson",
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "csv",
            ExportFormat::Json => "json",
            ExportFormat::Excel => "xlsx",
            ExportFormat::JsonLines => "jsonl",
        }
    }
}

/// Data to be exported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl ExportData {
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<serde_json::Value>) {
        self.rows.push(row);
    }

    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

/// Export trait
pub trait Exporter: Send + Sync {
    fn export(&self, data: &ExportData) -> ExportResult<Bytes>;
}

/// CSV Exporter
pub struct CsvExporter;

impl Exporter for CsvExporter {
    fn export(&self, data: &ExportData) -> ExportResult<Bytes> {
        let mut writer = csv::Writer::from_writer(Vec::new());

        // Write headers
        writer
            .write_record(&data.headers)
            .map_err(|e| ExportError::FormatError(format!("CSV header error: {}", e)))?;

        // Write rows
        for row in &data.rows {
            let string_row: Vec<String> = row
                .iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => String::new(),
                    _ => v.to_string(),
                })
                .collect();

            writer
                .write_record(&string_row)
                .map_err(|e| ExportError::FormatError(format!("CSV row error: {}", e)))?;
        }

        writer
            .flush()
            .map_err(|e| ExportError::FormatError(format!("CSV flush error: {}", e)))?;

        let bytes = writer
            .into_inner()
            .map_err(|e| ExportError::FormatError(format!("CSV finalize error: {}", e)))?;

        Ok(Bytes::from(bytes))
    }
}

/// JSON Exporter
pub struct JsonExporter;

impl Exporter for JsonExporter {
    fn export(&self, data: &ExportData) -> ExportResult<Bytes> {
        // Create JSON array of objects
        let mut records = Vec::new();

        for row in &data.rows {
            let mut record = serde_json::Map::new();
            for (i, header) in data.headers.iter().enumerate() {
                if let Some(value) = row.get(i) {
                    record.insert(header.clone(), value.clone());
                }
            }
            records.push(serde_json::Value::Object(record));
        }

        let output = serde_json::json!({
            "data": records,
            "metadata": data.metadata,
            "count": data.rows.len(),
        });

        let json_bytes = serde_json::to_vec_pretty(&output)?;
        Ok(Bytes::from(json_bytes))
    }
}

/// JSON Lines Exporter (NDJSON)
pub struct JsonLinesExporter;

impl Exporter for JsonLinesExporter {
    fn export(&self, data: &ExportData) -> ExportResult<Bytes> {
        let mut output = Vec::new();

        for row in &data.rows {
            let mut record = serde_json::Map::new();
            for (i, header) in data.headers.iter().enumerate() {
                if let Some(value) = row.get(i) {
                    record.insert(header.clone(), value.clone());
                }
            }

            let line = serde_json::to_string(&serde_json::Value::Object(record))?;
            writeln!(output, "{}", line)?;
        }

        Ok(Bytes::from(output))
    }
}

/// Excel Exporter
pub struct ExcelExporter {
    sheet_name: String,
}

impl ExcelExporter {
    pub fn new(sheet_name: impl Into<String>) -> Self {
        Self {
            sheet_name: sheet_name.into(),
        }
    }
}

impl Default for ExcelExporter {
    fn default() -> Self {
        Self::new("Data")
    }
}

impl Exporter for ExcelExporter {
    fn export(&self, data: &ExportData) -> ExportResult<Bytes> {
        use rust_xlsxwriter::*;

        let mut workbook = Workbook::new();
        let worksheet = workbook
            .add_worksheet()
            .set_name(&self.sheet_name)
            .map_err(|e| ExportError::FormatError(format!("Excel sheet error: {}", e)))?;

        // Header formatting
        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);

        // Write headers
        for (col, header) in data.headers.iter().enumerate() {
            worksheet
                .write_string_with_format(0, col as u16, header, &header_format)
                .map_err(|e| ExportError::FormatError(format!("Excel header error: {}", e)))?;
        }

        // Write data rows
        for (row_idx, row) in data.rows.iter().enumerate() {
            for (col_idx, value) in row.iter().enumerate() {
                let excel_row = (row_idx + 1) as u32;
                let excel_col = col_idx as u16;

                match value {
                    serde_json::Value::String(s) => {
                        worksheet
                            .write_string(excel_row, excel_col, s)
                            .map_err(|e| {
                                ExportError::FormatError(format!("Excel write error: {}", e))
                            })?;
                    }
                    serde_json::Value::Number(n) => {
                        if let Some(f) = n.as_f64() {
                            worksheet
                                .write_number(excel_row, excel_col, f)
                                .map_err(|e| {
                                    ExportError::FormatError(format!("Excel write error: {}", e))
                                })?;
                        } else {
                            worksheet
                                .write_string(excel_row, excel_col, &n.to_string())
                                .map_err(|e| {
                                    ExportError::FormatError(format!("Excel write error: {}", e))
                                })?;
                        }
                    }
                    serde_json::Value::Bool(b) => {
                        worksheet
                            .write_boolean(excel_row, excel_col, *b)
                            .map_err(|e| {
                                ExportError::FormatError(format!("Excel write error: {}", e))
                            })?;
                    }
                    serde_json::Value::Null => {
                        worksheet
                            .write_blank(excel_row, excel_col, &Format::new())
                            .map_err(|e| {
                                ExportError::FormatError(format!("Excel write error: {}", e))
                            })?;
                    }
                    _ => {
                        worksheet
                            .write_string(excel_row, excel_col, &value.to_string())
                            .map_err(|e| {
                                ExportError::FormatError(format!("Excel write error: {}", e))
                            })?;
                    }
                }
            }
        }

        // Auto-fit columns
        worksheet.autofit();

        // Save to bytes
        let bytes = workbook
            .save_to_buffer()
            .map_err(|e| ExportError::FormatError(format!("Excel save error: {}", e)))?;

        Ok(Bytes::from(bytes))
    }
}

/// Factory for creating exporters
pub fn create_exporter(format: ExportFormat) -> Box<dyn Exporter> {
    match format {
        ExportFormat::Csv => Box::new(CsvExporter),
        ExportFormat::Json => Box::new(JsonExporter),
        ExportFormat::Excel => Box::new(ExcelExporter::default()),
        ExportFormat::JsonLines => Box::new(JsonLinesExporter),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> ExportData {
        let mut data = ExportData::new(vec![
            "Name".to_string(),
            "Age".to_string(),
            "Active".to_string(),
        ]);

        data.add_row(vec![
            serde_json::json!("Alice"),
            serde_json::json!(30),
            serde_json::json!(true),
        ]);

        data.add_row(vec![
            serde_json::json!("Bob"),
            serde_json::json!(25),
            serde_json::json!(false),
        ]);

        data.add_metadata("exported_by", "test");
        data
    }

    #[test]
    fn test_csv_export() {
        let data = create_test_data();
        let exporter = CsvExporter;
        let result = exporter.export(&data);

        assert!(result.is_ok());
        let bytes = result.unwrap();
        let content = String::from_utf8(bytes.to_vec()).unwrap();

        assert!(content.contains("Name,Age,Active"));
        assert!(content.contains("Alice,30,true"));
    }

    #[test]
    fn test_json_export() {
        let data = create_test_data();
        let exporter = JsonExporter;
        let result = exporter.export(&data);

        assert!(result.is_ok());
        let bytes = result.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["count"], 2);
        assert_eq!(json["data"][0]["Name"], "Alice");
    }

    #[test]
    fn test_jsonlines_export() {
        let data = create_test_data();
        let exporter = JsonLinesExporter;
        let result = exporter.export(&data);

        assert!(result.is_ok());
        let bytes = result.unwrap();
        let content = String::from_utf8(bytes.to_vec()).unwrap();

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_excel_export() {
        let data = create_test_data();
        let exporter = ExcelExporter::default();
        let result = exporter.export(&data);

        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 0);
    }

    #[test]
    fn test_export_format_display() {
        assert_eq!(ExportFormat::Csv.to_string(), "csv");
        assert_eq!(ExportFormat::Json.to_string(), "json");
        assert_eq!(ExportFormat::Excel.to_string(), "xlsx");
    }

    #[test]
    fn test_export_format_mime_types() {
        assert_eq!(ExportFormat::Csv.mime_type(), "text/csv");
        assert_eq!(ExportFormat::Json.mime_type(), "application/json");
        assert_eq!(
            ExportFormat::Excel.mime_type(),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        );
    }
}
