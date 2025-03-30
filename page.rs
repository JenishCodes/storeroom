use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::vec::Vec;

use crate::datatype::DataType;
use crate::row::Row;

const PAGE_SIZE: usize = 4096;
const PAGE_HEADER_SIZE: usize = 16;
const PAGE_FILL_FACTOR: f32 = 0.8;

#[derive(Debug)]
pub struct Page {
    pub page_id: u32,
    pub page_type: u8,
    pub free_space_offset: u16,
    pub row_count: u16,
    pub checksum: u16,
    pub row_offset: Vec<u16>,
    pub is_dirty: bool,
    pub rows: Vec<Row>,

    pub columns: Vec<String>,
    pub schema: Vec<DataType>,
}

impl Page {
    pub fn new(page_id: u32, page_type: u8, columns: Vec<String>, schema: Vec<DataType>) -> Page {
        Page {
            page_id,
            page_type,
            free_space_offset: PAGE_HEADER_SIZE as u16,
            row_count: 0,
            checksum: 0,
            row_offset: Vec::new(),
            rows: Vec::new(),
            is_dirty: false,
            columns,
            schema,
        }
    }

    pub fn read(&mut self, file: &mut File) -> io::Result<()> {
        file.seek(SeekFrom::Start((self.page_id * PAGE_SIZE as u32) as u64))?;

        let mut buffer = [0; PAGE_SIZE];
        file.read_exact(&mut buffer)?;

        self.page_id = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        self.page_type = buffer[4];
        self.free_space_offset = u16::from_be_bytes([buffer[5], buffer[6]]);
        self.row_count = u16::from_be_bytes([buffer[7], buffer[8]]);
        self.checksum = u16::from_be_bytes([buffer[9], buffer[10]]);

        let mut i = 0;
        while i < 2 * self.row_count {
            let offset = u16::from_be_bytes([
                buffer[PAGE_HEADER_SIZE + i as usize],
                buffer[PAGE_HEADER_SIZE + i as usize + 1],
            ]);
            self.row_offset.push(offset);
            i += 2;
        }

        let mut last_row_offset = PAGE_SIZE as u16;
        for i in 0..self.row_count {
            let row_offset = self.row_offset[i as usize];
            let row_data = &mut buffer[row_offset as usize..last_row_offset as usize];

            let mut row = Row::new(self.columns.len());
            row.from_bytes(row_data, &self.schema);

            last_row_offset = row_offset;
            self.rows.push(row);
        }

        Ok(())
    }

    pub fn write(&mut self, file: &mut File) -> io::Result<()> {
        let mut buffer = [0; PAGE_SIZE];

        buffer[0..4].copy_from_slice(&self.page_id.to_be_bytes());
        buffer[4] = self.page_type;
        buffer[5..7].copy_from_slice(&self.free_space_offset.to_be_bytes());
        buffer[7..9].copy_from_slice(&self.row_count.to_be_bytes());
        buffer[9..11].copy_from_slice(&self.checksum.to_be_bytes());

        for (i, offset) in self.row_offset.iter().enumerate() {
            buffer[PAGE_HEADER_SIZE + i * 2..PAGE_HEADER_SIZE + (i + 1) * 2]
                .copy_from_slice(&offset.to_be_bytes());
        }

        let mut last_row_offset = PAGE_SIZE as u16;
        for (i, row) in self.rows.iter().enumerate() {
            let row_data = row.to_bytes(self.schema.as_ref());
            buffer[self.row_offset[i] as usize..last_row_offset as usize]
                .copy_from_slice(&row_data);
            last_row_offset = self.row_offset[i];
        }

        file.seek(SeekFrom::Start(self.page_id as u64 * PAGE_SIZE as u64))?;
        file.write_all(&buffer)?;

        self.is_dirty = false;
        
        Ok(())
    }

    pub fn add_row(&mut self, row: Row) -> io::Result<()> {
        let mut free_bytes = (PAGE_SIZE - PAGE_HEADER_SIZE) as u16;
        if self.row_count > 0 {
            free_bytes = self.free_space_offset - self.row_offset[self.row_count as usize - 1];
        }
        let row_data = row.to_bytes(self.schema.as_ref());
        let row_size = row_data.len();

        if free_bytes - (row_size as u16)
            < (PAGE_SIZE as f32 * (1 as f32 - PAGE_FILL_FACTOR)) as u16
        {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Not enough space to add row",
            ));
        }

        self.rows.push(row);
        self.row_offset.push(
            if self.row_count == 0 {
                PAGE_SIZE as u16
            } else {
                self.row_offset[self.row_count as usize - 1]
            } - row_size as u16,
        );
        self.free_space_offset += 2;
        self.row_count += 1;

        self.is_dirty = true;

        Ok(())
    }

    pub fn print(&self) {
        println!("Page ID: {}", self.page_id);
        println!("Page Type: {}", self.page_type);
        println!("Free Space Offset: {}", self.free_space_offset);
        println!("Row Count: {}", self.row_count);
        println!("Checksum: {}", self.checksum);
        println!("Row Offsets: {:?}", self.row_offset);
        println!("Columns: {:?}", self.columns);
        println!("Schema: {:?}", self.schema);
        println!("Is Dirty: {}", self.is_dirty);
        println!("\nRows:");
        for row in &self.rows {
            println!("---------------------");
            row.print(&self.schema, &self.columns);
            println!("---------------------");
        }
    }
}
