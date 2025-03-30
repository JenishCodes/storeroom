use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

#[derive(Debug, Clone)]
pub enum DataType {
    Number(i32),
    Float(f32),
    DateTime(NaiveDateTime),
    Date(NaiveDate),
    Text(String),
    Boolean(bool),
}

impl DataType {
    pub fn new(data_type: &str) -> DataType {
        match data_type.to_lowercase().as_str() {
            "number" => DataType::Number(0),
            "float" => DataType::Float(0.0),
            "datetime" => DataType::DateTime(
                NaiveDateTime::parse_from_str("1970-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            ),
            "date" => DataType::Date(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
            "text" => DataType::Text("".to_string()),
            "boolean" => DataType::Boolean(false),
            _ => panic!("Unknown data type"),
        }
    }

    pub fn to_u8(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            DataType::Number(value) => {
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            DataType::Float(value) => {
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            DataType::Boolean(value) => {
                bytes.push(if *value { 1 } else { 0 });
            }
            DataType::Text(value) => {
                let length = value.len() as u16;
                bytes.extend_from_slice(&length.to_be_bytes());
                bytes.extend_from_slice(value.as_bytes());
            }
            DataType::Date(value) => {
                let base_date = NaiveDate::from_ymd_opt(1970, 1, 1).expect("Invalid date");
                let days_since: i32 = value.signed_duration_since(base_date).num_days() as i32;
                bytes.extend_from_slice(&days_since.to_be_bytes());
            }
            DataType::DateTime(value) => {
                let base_date = NaiveDate::from_ymd_opt(1970, 1, 1).expect("Invalid date");
                let days_since: i32 =
                    value.date().signed_duration_since(base_date).num_days() as i32;
                let time = value.time();
                let seconds_since: i32 =
                    (time.hour() * 3600 + time.minute() * 60 + time.second()) as i32;
                bytes.extend_from_slice(&days_since.to_be_bytes());
                bytes.extend_from_slice(&seconds_since.to_be_bytes());
            }
        };

        bytes
    }

    pub fn from_u8(&self, buffer: &[u8]) -> Result<(DataType, usize), String> {
        match self {
            DataType::Number(_) => {
                if buffer.len() < 4 {
                    return Err(format!("Expected 4 bytes, got {}", buffer.len()));
                }

                let number: i32 = i32::from_be_bytes(buffer[..4].try_into().unwrap());
                Ok((DataType::Number(number), 4))
            }
            DataType::Float(_) => {
                if buffer.len() < 4 {
                    return Err(format!("Expected 4 bytes, got {}", buffer.len()));
                }

                let float: f32 = f32::from_be_bytes(buffer[..4].try_into().unwrap());
                Ok((DataType::Float(float), 4))
            }
            DataType::Boolean(_) => {
                if buffer.len() < 1 {
                    return Err(format!("Expected 1 byte, got {}", buffer.len()));
                }

                let boolean: [u8; 1] = buffer[..1].try_into().unwrap();
                Ok((DataType::Boolean(boolean[0] == 1), 1))
            }
            DataType::Text(_) => {
                if buffer.len() < 2 {
                    return Err(format!("Expected 2 bytes, got {}", buffer.len()));
                }
                let length = u16::from_be_bytes(buffer[..2].try_into().unwrap()) as usize;

                if buffer.len() < length + 2 {
                    return Err(format!(
                        "Expected {} bytes, got {}",
                        length + 2,
                        buffer.len()
                    ));
                }

                let text =
                    String::from_utf8(buffer[2..length + 2].to_vec()).map_err(|e| e.to_string())?;
                Ok((DataType::Text(text), length + 2))
            }
            DataType::Date(_) => {
                if buffer.len() < 4 {
                    return Err(format!("Expected 4 bytes, got {}", buffer.len()));
                }
                let days_since = i32::from_be_bytes(buffer[..4].try_into().unwrap());

                let date = NaiveDate::from_ymd_opt(1970, 1, 1).expect("Invalid date")
                    + Duration::days(days_since as i64);

                Ok((DataType::Date(date), 4))
            }
            DataType::DateTime(_) => {
                if buffer.len() < 8 {
                    return Err(format!("Expected 8 bytes, got {}", buffer.len()));
                }

                let days_since = i32::from_be_bytes(buffer[..4].try_into().unwrap());
                let seconds_since = u32::from_be_bytes(buffer[4..8].try_into().unwrap());

                let base_date = NaiveDate::from_ymd_opt(1970, 1, 1).expect("Invalid date");

                let date = base_date + Duration::days(days_since as i64);
                let time = NaiveTime::from_num_seconds_from_midnight_opt(seconds_since, 0)
                    .expect("Invalid time");

                Ok((DataType::DateTime(NaiveDateTime::new(date, time)), 8))
            }
        }
    }
}
