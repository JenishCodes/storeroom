use crate::datatype::DataType;

#[derive(Debug, Clone)]
pub struct Row {
    pub null_map: Vec<bool>,
    pub values: Vec<DataType>,
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

                let (value, length) = schema[i].from_u8(&buffer[offset..]).unwrap();

                self.values.push(value);
                offset += length;
            } else {
                self.null_map[i] = true;
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = vec![0; 8];
        let mut j = 0;
        for i in 0..self.null_map.len() {
            if self.null_map[i] {
                buffer[i] = 1;
            } else {
                let bytes = self.values[j].to_u8();
                buffer.extend_from_slice(&bytes);
                j += 1;
            }
        }
        buffer
    }

    pub fn print(&self, columns: &Vec<String>) {
        let mut j = 0;
        for i in 0..self.null_map.len() {
            if self.null_map[i] {
                println!("{}: NULL", columns[i]);
            } else {
                println!("{}: {:?}", columns[i], self.values[j]);
                j += 1;
            }
        }
    }
}
