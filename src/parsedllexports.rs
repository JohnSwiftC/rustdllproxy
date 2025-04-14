use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug)]
pub enum DllParserError {
    IoError(io::Error),
    InvalidPeFormat,
    UnsupportedFormat,
}

impl From<io::Error> for DllParserError {
    fn from(error: io::Error) -> Self {
        DllParserError::IoError(error)
    }
}

pub fn parse_dll_exports<P: AsRef<Path>>(dll_path: P) -> Result<Vec<String>, DllParserError> {
    let mut file = File::open(dll_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if buffer.len() < 0x40 || buffer[0] != b'M' || buffer[1] != b'Z' {
        return Err(DllParserError::InvalidPeFormat);
    }

    let pe_offset = u32::from_le_bytes([buffer[0x3C], buffer[0x3D], buffer[0x3E], buffer[0x3F]]) as usize;
    
    if buffer.len() < pe_offset + 4 || 
       buffer[pe_offset] != b'P' || 
       buffer[pe_offset + 1] != b'E' || 
       buffer[pe_offset + 2] != 0 || 
       buffer[pe_offset + 3] != 0 {
        return Err(DllParserError::InvalidPeFormat);
    }

    let machine_type = u16::from_le_bytes([buffer[pe_offset + 4], buffer[pe_offset + 5]]);
    let is_64bit = machine_type == 0x8664;
    
    let num_sections = u16::from_le_bytes([buffer[pe_offset + 6], buffer[pe_offset + 7]]) as usize;
    
    let optional_header_size = u16::from_le_bytes([buffer[pe_offset + 20], buffer[pe_offset + 21]]) as usize;
    
    let optional_header_offset = pe_offset + 24;
    
    let optional_header_magic = u16::from_le_bytes([
        buffer[optional_header_offset], 
        buffer[optional_header_offset + 1]
    ]);
    
    if (is_64bit && optional_header_magic != 0x20B) || 
       (!is_64bit && optional_header_magic != 0x10B) {
        return Err(DllParserError::InvalidPeFormat);
    }
    
    let export_dir_rva_offset = if is_64bit { 
        optional_header_offset + 112
    } else {
        optional_header_offset + 96
    };
    
    if buffer.len() < export_dir_rva_offset + 8 {
        return Err(DllParserError::InvalidPeFormat);
    }
    
    let export_dir_rva = u32::from_le_bytes([
        buffer[export_dir_rva_offset],
        buffer[export_dir_rva_offset + 1],
        buffer[export_dir_rva_offset + 2],
        buffer[export_dir_rva_offset + 3],
    ]);
    
    let export_dir_size = u32::from_le_bytes([
        buffer[export_dir_rva_offset + 4],
        buffer[export_dir_rva_offset + 5],
        buffer[export_dir_rva_offset + 6],
        buffer[export_dir_rva_offset + 7],
    ]);
    
    if export_dir_rva == 0 || export_dir_size == 0 {
        return Ok(Vec::new());
    }
    
    let section_header_offset = optional_header_offset + optional_header_size;
    
    let export_dir_file_offset = rva_to_file_offset(
        &buffer, 
        section_header_offset, 
        num_sections, 
        export_dir_rva
    )?;
    
    if buffer.len() < export_dir_file_offset + 24 + 4 {
        return Err(DllParserError::InvalidPeFormat);
    }
    
    let number_of_names = u32::from_le_bytes([
        buffer[export_dir_file_offset + 24],
        buffer[export_dir_file_offset + 25],
        buffer[export_dir_file_offset + 26],
        buffer[export_dir_file_offset + 27],
    ]) as usize;
    
    if buffer.len() < export_dir_file_offset + 32 + 4 {
        return Err(DllParserError::InvalidPeFormat);
    }
    
    let name_pointer_rva = u32::from_le_bytes([
        buffer[export_dir_file_offset + 32],
        buffer[export_dir_file_offset + 33],
        buffer[export_dir_file_offset + 34],
        buffer[export_dir_file_offset + 35],
    ]);
    
    let name_pointer_file_offset = rva_to_file_offset(
        &buffer, 
        section_header_offset, 
        num_sections, 
        name_pointer_rva
    )?;
    
    let mut export_names = Vec::with_capacity(number_of_names);
    
    for i in 0..number_of_names {
        let name_rva_offset = name_pointer_file_offset + (i * 4);
        
        if buffer.len() < name_rva_offset + 4 {
            return Err(DllParserError::InvalidPeFormat);
        }
        
        let name_rva = u32::from_le_bytes([
            buffer[name_rva_offset],
            buffer[name_rva_offset + 1],
            buffer[name_rva_offset + 2],
            buffer[name_rva_offset + 3],
        ]);
        
        let name_file_offset = rva_to_file_offset(
            &buffer, 
            section_header_offset, 
            num_sections, 
            name_rva
        )?;
        
        let mut name = Vec::new();
        let mut offset = name_file_offset;
        
        while offset < buffer.len() && buffer[offset] != 0 {
            name.push(buffer[offset]);
            offset += 1;
        }
        
        match String::from_utf8(name) {
            Ok(export_name) => export_names.push(export_name),
            Err(_) => continue,
        }
    }
    
    Ok(export_names)
}

fn rva_to_file_offset(
    buffer: &[u8], 
    section_header_offset: usize, 
    num_sections: usize, 
    rva: u32
) -> Result<usize, DllParserError> {
    for i in 0..num_sections {
        let section_offset = section_header_offset + (i * 40);
        
        if buffer.len() < section_offset + 40 {
            return Err(DllParserError::InvalidPeFormat);
        }
        
        let section_rva = u32::from_le_bytes([
            buffer[section_offset + 12],
            buffer[section_offset + 13],
            buffer[section_offset + 14],
            buffer[section_offset + 15],
        ]);
        
        let section_size = u32::from_le_bytes([
            buffer[section_offset + 16],
            buffer[section_offset + 17],
            buffer[section_offset + 18],
            buffer[section_offset + 19],
        ]);
        
        let section_pointer_to_raw_data = u32::from_le_bytes([
            buffer[section_offset + 20],
            buffer[section_offset + 21],
            buffer[section_offset + 22],
            buffer[section_offset + 23],
        ]);
        
        if rva >= section_rva && rva < section_rva + section_size {
            let offset = (rva - section_rva) + section_pointer_to_raw_data;
            return Ok(offset as usize);
        }
    }
    
    Err(DllParserError::UnsupportedFormat)
}