use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

#[derive(Debug, Clone, Copy)]
pub enum DataType {
    Number,
    Float,
    DateTime,
    Date,
    Text,
    Boolean,
}

impl DataType {
    pub fn to_u8(&self, value: &Box<dyn std::any::Any>) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            DataType::Number => {
                let value = value.downcast_ref::<i32>().unwrap();
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            DataType::Float => {
                let value = value.downcast_ref::<f32>().unwrap();
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            DataType::Boolean => {
                let value = value.downcast_ref::<bool>().unwrap();
                bytes.push(if *value { 1 } else { 0 });
            }
            DataType::Text => {
                let value = value.downcast_ref::<String>().unwrap();
                let length = value.len() as u16;
                bytes.extend_from_slice(&length.to_be_bytes());
                bytes.extend_from_slice(value.as_bytes());
            }
            DataType::Date => {
                let value = value.downcast_ref::<NaiveDate>().unwrap();
                let base_date = NaiveDate::from_ymd_opt(1970, 1, 1).expect("Invalid date");
                let days_since: u32 = value.signed_duration_since(base_date).num_days() as u32;
                bytes.extend_from_slice(&days_since.to_be_bytes());
            }
            DataType::DateTime => {
                let value = value.downcast_ref::<NaiveDateTime>().unwrap();
                let base_date = NaiveDate::from_ymd_opt(1970, 1, 1).expect("Invalid date");
                let days_since: u32 =
                    value.date().signed_duration_since(base_date).num_days() as u32;
                let time = value.time();
                let seconds_since: u32 =
                    (time.hour() * 3600 + time.minute() * 60 + time.second()) as u32;
                bytes.extend_from_slice(&days_since.to_be_bytes());
                bytes.extend_from_slice(&seconds_since.to_be_bytes());
            }
        };

        bytes
    }

    pub fn from_u8(&self, buffer: &[u8]) -> Result<(Box<dyn std::any::Any>, usize), String> {
        match self {
            DataType::Number => {
                if buffer.len() < 4 {
                    return Err(format!(
                        "Expected 4 bytes for Integer, got {}",
                        buffer.len()
                    ));
                }
                let bytes: [u8; 4] = buffer[..4].try_into().unwrap();
                Ok((Box::new(i32::from_be_bytes(bytes)), 4))
            }
            DataType::Float => {
                if buffer.len() < 4 {
                    return Err(format!("Expected 4 bytes for Float, got {}", buffer.len()));
                }
                let bytes: [u8; 4] = buffer[..4].try_into().unwrap();
                Ok((
                    Box::new(f32::from_be_bytes(bytes[..4].try_into().unwrap())),
                    4,
                ))
            }
            DataType::Boolean => {
                if buffer.len() < 1 {
                    return Err(format!("Expected 1 byte for Boolean, got {}", buffer.len()));
                }
                let bytes: [u8; 1] = buffer[..1].try_into().unwrap();
                Ok((Box::new(bytes[0] == 1), 1))
            }
            DataType::Text => {
                if buffer.len() < 3 {
                    return Err(format!("Expected 3 byte for Boolean, got {}", buffer.len()));
                }
                let length_bytes: [u8; 2] = buffer[..2].try_into().unwrap();
                let length = u16::from_be_bytes(length_bytes) as usize;

                if buffer.len() < length {
                    return Err(format!(
                        "Expected {} bytes for Text, got {}",
                        length,
                        buffer.len()
                    ));
                }
                let bytes: Vec<u8> = buffer[2..length + 2].to_vec();
                let text = String::from_utf8(bytes.to_vec()).unwrap();

                Ok((Box::new(text), length + 2))
            }
            DataType::Date => {
                if buffer.len() < 4 {
                    return Err(format!("Expected 4 bytes for Date, got {}", buffer.len()));
                }
                let bytes: [u8; 4] = buffer[..4].try_into().unwrap();
                let days_since = i32::from_be_bytes(bytes);

                Ok((
                    Box::new(
                        NaiveDate::from_ymd_opt(1970, 1, 1).expect("Invalid date")
                            + Duration::days(days_since as i64),
                    ),
                    4,
                ))
            }
            DataType::DateTime => {
                if buffer.len() < 8 {
                    return Err(format!(
                        "Expected 8 bytes for DateTime, got {}",
                        buffer.len()
                    ));
                }
                let bytes: [u8; 8] = buffer[..8].try_into().unwrap();

                let days_since = i32::from_be_bytes(bytes[0..4].try_into().unwrap());
                let seconds_since = u32::from_be_bytes(bytes[4..8].try_into().unwrap());

                let base_date = NaiveDate::from_ymd_opt(1970, 1, 1).expect("Invalid date");

                let date = base_date + Duration::days(days_since as i64);
                let time = NaiveTime::from_num_seconds_from_midnight_opt(seconds_since, 0)
                    .expect("Invalid time");

                Ok((Box::new(NaiveDateTime::new(date, time)), 8))
            }
        }
    }
}
