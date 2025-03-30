use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::vec::Vec;

const PAGE_SIZE: usize = 4096;
const PAGE_HEADER_SIZE: usize = 16;
const DEFAULT_ROW_OFFSET_SIZE: usize = 100;
const PAGE_FILL_FACTOR: f32 = 0.8;

#[derive(Debug)]
struct Page {
    page_id: u32,
    page_type: u8,
    free_space_offset: u16,
    row_count: u16,
    checksum: u16,
    row_offset: Vec<u16>,
    data: Vec<u8>,
}

impl Page {
    fn read(file: &mut File, page_offset: u64) -> Page {
        file.seek(SeekFrom::Start(page_offset))?;

        let mut buffer = [0; PAGE_SIZE];
        file.read_exact(&buffer)?;

        let page_id = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let page_type = buffer[4];
        let free_space_offset = u16::from_le_bytes([buffer[5], buffer[6]]);
        let row_count = u16::from_le_bytes([buffer[7], buffer[8]]);
        let checksum = u16::from_le_bytes([buffer[9], buffer[10]]);

        let row_offset_length = row_count * 2 as usize;
        let mut row_offset = Vec::with_capacity(row_offset_length);

        let mut i = 0;
        while i < 2 * row_offset_length {
            let offset = u16::from_le_bytes([buffer[PAGE_HEADER_SIZE + i], buffer[PAGE_HEADER_SIZE + i + 1]]);
            row_offset.push(offset);
            i += 2;
        }

        let mut data = Vec::with_capacity(row_count);
        

        
    }
}
