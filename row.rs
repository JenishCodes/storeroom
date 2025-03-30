use crate::datatype::DataType;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Row {
    pub null_map: Vec<bool>,
    pub columns: Vec<String>,
    pub schema: Vec<DataType>,
    pub values: Vec<Box<dyn std::any::Any>>,
}

impl Row {
    pub fn new(row_schema: HashMap<&str, &DataType>) -> Row {
        let mut columns = Vec::new();
        let mut schema = Vec::new();

        for (name, data_type) in row_schema {
            columns.push(name.to_string());
            schema.push(data_type.clone());
        }

        Row {
            null_map: vec![true; columns.len()],
            values: Vec::with_capacity(columns.len()),
            columns,
            schema,
        }
    }

    pub fn read(&mut self, buffer: &mut [u8]) {
        let mut offset = 8;
        for i in 0..self.columns.len() {
            if buffer[i] == 0 {
                self.null_map[i] = false;

                let data_type = &self.schema[i];
                let (value, length) = data_type.from_u8(&buffer[offset..]).unwrap();
                self.values.push(value);
                offset += length;
            } else {
                self.null_map[i] = true;
            }
        }
    }
}
