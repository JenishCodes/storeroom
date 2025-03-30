use crate::datatype::DataType;
use chrono::{NaiveDate, NaiveDateTime};


#[derive(Debug)]
pub struct Row {
    pub null_map: Vec<bool>,
    pub values: Vec<Box<dyn std::any::Any>>,
}

impl Row {
    pub fn new(column_count: usize) -> Row {
        Row {
            null_map: vec![true; column_count],
            values: Vec::with_capacity(column_count),
        }
    }

    pub fn from_bytes(&mut self, buffer: &mut [u8], schema: &Vec<DataType>) {
        let mut offset = 8;
        for i in 0..self.null_map.len() {
            if buffer[i] == 0 {
                self.null_map[i] = false;

                let data_type = schema[i];
                let (value, length) = data_type.from_u8(&buffer[offset..]).unwrap();
                self.values.push(value);
                offset += length;
            } else {
                self.null_map[i] = true;
            }
        }
    }

    pub fn to_bytes(&self, schema: &Vec<DataType>) -> Vec<u8> {
        let mut buffer = vec![0; 8];
        let mut offset = 0;
        for i in 0..self.null_map.len() {
            if self.null_map[i] {
                buffer[i] = 1;
            } else {
                let data_type = schema[i];
                let value = &self.values[offset];

                let bytes = data_type.to_u8(value);
                buffer.extend_from_slice(&bytes);
                offset += 1;
            }
        }
        buffer
    }

    pub fn print(&self, schema: &Vec<DataType>, columns: &Vec<String>) {
        for (i, value) in self.values.iter().enumerate() {
            if self.null_map[i] {
                println!("{}: NULL", columns[i]);
            } else {
                match schema[i] {
                    DataType::Number => {
                        let value = value.downcast_ref::<i32>().unwrap();
                        println!("{}: {}", columns[i], value);
                    }
                    DataType::Float => {
                        let value = value.downcast_ref::<f32>().unwrap();
                        println!("{}: {}", columns[i], value);
                    }
                    DataType::Boolean => {
                        let value = value.downcast_ref::<bool>().unwrap();
                        println!("{}: {}", columns[i], value);
                    }
                    DataType::Text => {
                        let value = value.downcast_ref::<String>().unwrap();
                        println!("{}: {}", columns[i], value);
                    }
                    DataType::Date => {
                        let value = value.downcast_ref::<NaiveDate>().unwrap();
                        println!("{}: {}", columns[i], value);
                    }
                    DataType::DateTime => {
                        let value = value.downcast_ref::<NaiveDateTime>().unwrap();
                        println!("{}: {}", columns[i], value);
                    }
                }
            }
        }
    }
}
