use csv::WriterBuilder;
use serde_json::Value as JsonValue;
use std::error::Error;

pub async fn save_json_as_csv(
    records: Vec<JsonValue>,
    csv_file_path: &str,
) -> Result<(), Box<dyn Error>> {
    // Create a CSV writer
    let mut wtr = WriterBuilder::new()
        .delimiter(b';')
        .from_path(csv_file_path)?;

    // Initialize headers
    let mut headers: Vec<String> = Vec::new();

    // Write headers
    if let Some(first_record) = records.get(0) {
        if let Some(object) = first_record.as_object() {
            // Write headers based on the keys of the first object
            headers = object.keys().cloned().collect();
            wtr.write_record(&headers)?;
        }
    }

    // Write the records to the CSV file
    for record in records {
        if let Some(object) = record.as_object() {
            let row: Vec<String> = headers
                .iter()
                .map(|key| {
                    // Get the value for the current key and convert it to a string
                    match object.get(key) {
                        Some(value) => match value {
                            JsonValue::String(s) => s.clone(),
                            JsonValue::Number(n) => n.to_string(),
                            JsonValue::Bool(b) => b.to_string(),
                            JsonValue::Null => "".to_string(),
                            _ => "".to_string(), // Handle other types if necessary
                        },
                        None => "".to_string(), // Key not found
                    }
                })
                .collect();
            wtr.write_record(&row)?;
        }
    }

    // Flush and finalize the CSV writer
    wtr.flush()?;
    Ok(())
}
