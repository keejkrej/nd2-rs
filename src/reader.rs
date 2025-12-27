use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use crate::chunk::{read_chunk, read_chunkmap, ChunkMap};
use crate::constants::{JP2_MAGIC, ND2_CHUNK_MAGIC, ND2_FILE_SIGNATURE};
use crate::error::{Nd2Error, Result};
use crate::metadata::{parse_attributes, parse_experiment, parse_text_info};
use crate::parse::ClxLiteParser;
use crate::types::{Attributes, ExpLoop, TextInfo};

/// Main reader for ND2 files
pub struct Nd2File {
    reader: BufReader<File>,
    version: (u32, u32),
    chunkmap: ChunkMap,
    // Cached metadata
    attributes: Option<Attributes>,
    experiment: Option<Vec<ExpLoop>>,
    text_info: Option<TextInfo>,
}

impl Nd2File {
    /// Open an ND2 file for reading
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read and validate file header
        let version = Self::read_version(&mut reader)?;

        // Validate version is supported (2.0, 2.1, 3.0)
        if version.0 < 2 || version.0 > 3 {
            return Err(Nd2Error::UnsupportedVersion {
                major: version.0,
                minor: version.1,
            });
        }

        // Read chunkmap from end of file
        let chunkmap = read_chunkmap(&mut reader)?;

        Ok(Self {
            reader,
            version,
            chunkmap,
            attributes: None,
            experiment: None,
            text_info: None,
        })
    }

    /// Get the file format version (major, minor)
    pub fn version(&self) -> (u32, u32) {
        self.version
    }

    /// Get image attributes
    pub fn attributes(&mut self) -> Result<&Attributes> {
        if self.attributes.is_none() {
            let chunk_name: &[u8] = if self.version.0 >= 3 {
                b"ImageAttributesLV!"
            } else {
                b"ImageAttributes!"
            };
            let data = read_chunk(&mut self.reader, &self.chunkmap, chunk_name)?;
            let parser = ClxLiteParser::new(false);
            let clx = parser.parse(&data)?;
            self.attributes = Some(parse_attributes(clx)?);
        }
        Ok(self.attributes.as_ref().unwrap())
    }

    /// Get experiment loop definitions
    pub fn experiment(&mut self) -> Result<&Vec<ExpLoop>> {
        if self.experiment.is_none() {
            let chunk_name: &[u8] = if self.version.0 >= 3 {
                b"ImageMetadataLV!"
            } else {
                b"ImageMetadata!"
            };

            if !self.chunkmap.contains_key(chunk_name) {
                self.experiment = Some(Vec::new());
            } else {
                let data = read_chunk(&mut self.reader, &self.chunkmap, chunk_name)?;
                let parser = ClxLiteParser::new(false);
                let clx = parser.parse(&data)?;
                self.experiment = Some(parse_experiment(clx)?);
            }
        }
        Ok(self.experiment.as_ref().unwrap())
    }

    /// Get text info (descriptions, author, date, etc.)
    pub fn text_info(&mut self) -> Result<&TextInfo> {
        if self.text_info.is_none() {
            let chunk_name: &[u8] = if self.version.0 >= 3 {
                b"ImageTextInfoLV!"
            } else {
                b"ImageTextInfo!"
            };

            if !self.chunkmap.contains_key(chunk_name) {
                self.text_info = Some(TextInfo::default());
            } else {
                let data = read_chunk(&mut self.reader, &self.chunkmap, chunk_name)?;
                let parser = ClxLiteParser::new(false);
                let clx = parser.parse(&data)?;
                self.text_info = Some(parse_text_info(clx)?);
            }
        }
        Ok(self.text_info.as_ref().unwrap())
    }

    /// List all chunk names in the file
    pub fn chunk_names(&self) -> Vec<String> {
        self.chunkmap
            .keys()
            .filter_map(|k| String::from_utf8(k.clone()).ok())
            .collect()
    }

    /// Read raw chunk data by name
    pub fn read_raw_chunk(&mut self, name: &[u8]) -> Result<Vec<u8>> {
        read_chunk(&mut self.reader, &self.chunkmap, name)
    }

    fn read_version<R: Read + Seek>(reader: &mut R) -> Result<(u32, u32)> {
        reader.seek(SeekFrom::Start(0))?;

        let mut header = [0u8; 112]; // 4 + 4 + 8 + 32 + 64
        reader.read_exact(&mut header)?;

        let magic = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);

        if magic == JP2_MAGIC {
            return Ok((1, 0)); // Legacy format
        }

        if magic != ND2_CHUNK_MAGIC {
            return Err(Nd2Error::InvalidMagic {
                expected: ND2_CHUNK_MAGIC,
                actual: magic,
            });
        }

        let name_length = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        let data_length = u64::from_le_bytes([
            header[8], header[9], header[10], header[11], header[12], header[13], header[14],
            header[15],
        ]);

        // Validate header
        if name_length != 32 || data_length != 64 {
            return Err(Nd2Error::InvalidFormat(
                "Corrupt file header".to_string(),
            ));
        }

        // Check signature
        let name = &header[16..48];
        if name != ND2_FILE_SIGNATURE {
            return Err(Nd2Error::InvalidFormat(
                "Invalid file signature".to_string(),
            ));
        }

        // Parse version from data (e.g., "Ver3.0")
        let data = &header[48..112];
        let major = (data[3] as char).to_digit(10).unwrap_or(0);
        let minor = (data[5] as char).to_digit(10).unwrap_or(0);

        Ok((major, minor))
    }
}

impl Drop for Nd2File {
    fn drop(&mut self) {
        // File is automatically closed when BufReader<File> is dropped
    }
}
