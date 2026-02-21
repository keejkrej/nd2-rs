use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use flate2::read::ZlibDecoder;

use crate::chunk::{read_chunk, read_chunkmap, ChunkMap};
use crate::constants::{JP2_MAGIC, ND2_CHUNK_MAGIC, ND2_FILE_SIGNATURE};
use crate::error::{Nd2Error, Result};
use crate::meta_parse::{parse_attributes, parse_experiment, parse_text_info};
use crate::parse::ClxLiteParser;
use crate::types::{Attributes, CompressionType, ExpLoop, TextInfo};

/// Axis names matching nd2-py AXIS
const AXIS_T: &str = "T";
const AXIS_P: &str = "P";
const AXIS_C: &str = "C";
const AXIS_Z: &str = "Z";
const AXIS_Y: &str = "Y";
const AXIS_X: &str = "X";

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
                // v3 wraps in SLxExperiment; unwrap if present and is object
                let to_parse = if self.version.0 >= 3 {
                    match clx.as_object().and_then(|o| o.get("SLxExperiment")) {
                        Some(inner) if inner.as_object().is_some() => inner.clone(),
                        _ => clx,
                    }
                } else {
                    clx
                };
                self.experiment = Some(parse_experiment(to_parse).unwrap_or_default());
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

    /// Dimensions (P,T,C,Z,Y,X) derived from attributes + experiment.
    /// When experiment is empty, infers minimal structure from sequence_count.
    pub fn sizes(&mut self) -> Result<HashMap<String, usize>> {
        let attrs = self.attributes()?.clone();
        let exp = self.experiment()?.clone();

        let n_chan = attrs.channel_count.unwrap_or(attrs.component_count);
        let height = attrs.height_px as usize;
        let width = attrs.width_px.or(attrs.width_bytes.map(|w| {
            let bpp = attrs.bits_per_component_in_memory / 8;
            w / (bpp * attrs.component_count)
        })).unwrap_or(0) as usize;

        let mut sizes: HashMap<String, usize> = HashMap::new();

        if exp.is_empty() {
            // Fallback: assume P=1, Z=1, infer T from sequence_count
            let total = attrs.sequence_count as usize;
            let n_z: usize = 1;
            let n_pos: usize = 1;
            let n_chan_usize = n_chan as usize;
            let n_time = total / (n_pos * n_chan_usize * n_z).max(1);
            sizes.insert(AXIS_P.to_string(), n_pos);
            sizes.insert(AXIS_T.to_string(), n_time);
            sizes.insert(AXIS_C.to_string(), n_chan_usize);
            sizes.insert(AXIS_Z.to_string(), n_z);
        } else {
            for loop_ in exp {
                match loop_ {
                    ExpLoop::TimeLoop(t) => {
                        sizes.insert(AXIS_T.to_string(), t.count as usize);
                    }
                    ExpLoop::XYPosLoop(xy) => {
                        sizes.insert(AXIS_P.to_string(), xy.count as usize);
                    }
                    ExpLoop::ZStackLoop(z) => {
                        sizes.insert(AXIS_Z.to_string(), z.count as usize);
                    }
                    ExpLoop::NETimeLoop(n) => {
                        sizes.insert(AXIS_T.to_string(), n.count as usize);
                    }
                    ExpLoop::CustomLoop(_) => {}
                }
            }
            if !sizes.contains_key(AXIS_C) {
                sizes.insert(AXIS_C.to_string(), n_chan as usize);
            }
            if !sizes.contains_key(AXIS_P) {
                sizes.insert(AXIS_P.to_string(), 1);
            }
            if !sizes.contains_key(AXIS_T) {
                sizes.insert(AXIS_T.to_string(), 1);
            }
            if !sizes.contains_key(AXIS_Z) {
                sizes.insert(AXIS_Z.to_string(), 1);
            }
        }

        sizes.insert(AXIS_Y.to_string(), height);
        sizes.insert(AXIS_X.to_string(), width);

        Ok(sizes)
    }

    /// Loop indices for each frame: seq_index -> (P, T, C, Z) as axis name -> index.
    pub fn loop_indices(&mut self) -> Result<Vec<HashMap<String, usize>>> {
        let sizes = self.sizes()?;
        let n_pos = *sizes.get(AXIS_P).unwrap_or(&1);
        let n_time = *sizes.get(AXIS_T).unwrap_or(&1);
        let n_chan = *sizes.get(AXIS_C).unwrap_or(&1);
        let n_z = *sizes.get(AXIS_Z).unwrap_or(&1);

        // Row-major order matching nd2-py: P (outer), T, C, Z (inner)
        let total = n_pos * n_time * n_chan * n_z;
        let mut out = Vec::with_capacity(total);

        for seq in 0..total {
            let mut idx = seq;
            let z = idx % n_z;
            idx /= n_z;
            let c = idx % n_chan;
            idx /= n_chan;
            let t = idx % n_time;
            idx /= n_time;
            let p = idx % n_pos;

            let mut m = HashMap::new();
            m.insert(AXIS_P.to_string(), p);
            m.insert(AXIS_T.to_string(), t);
            m.insert(AXIS_C.to_string(), c);
            m.insert(AXIS_Z.to_string(), z);
            out.push(m);
        }

        Ok(out)
    }

    /// Read one frame by sequence index. Returns pixels as (C, Y, X) u16 data.
    pub fn read_frame(&mut self, index: usize) -> Result<Vec<u16>> {
        let attrs = self.attributes()?.clone();
        let chunk_name = format!("ImageDataSeq|{}!", index);
        let chunk_key = chunk_name.as_bytes();

        let h = attrs.height_px as usize;
        let w = attrs.width_px.unwrap_or(0) as usize;
        let (n_c, n_comp) = match attrs.channel_count {
            Some(ch) if ch > 0 => (
                ch as usize,
                (attrs.component_count / ch) as usize,
            ),
            _ => (attrs.component_count as usize, 1),
        };
        let frame_size = h * w * n_c * n_comp;
        let expected_raw = frame_size * (attrs.bits_per_component_in_memory / 8) as usize;

        let data = self.read_raw_chunk(chunk_key)?;

        let pixel_bytes = if attrs.compression_type == Some(CompressionType::Lossless) {
            let mut decoder = ZlibDecoder::new(&data[8..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            decompressed
        } else if data.len() == expected_raw {
            data
        } else if data.len() >= 8 && (data.len() - 8) == expected_raw {
            data[8..].to_vec()
        } else {
            data
        };

        if pixel_bytes.len() / 2 < frame_size {
            return Err(Nd2Error::InvalidFormat(format!(
                "Frame {}: expected {} pixels ({} bytes), got {} bytes",
                index,
                frame_size,
                frame_size * 2,
                pixel_bytes.len()
            )));
        }

        let mut pixels: Vec<u16> = vec![0; pixel_bytes.len() / 2];
        for (i, chunk) in pixel_bytes.chunks_exact(2).enumerate() {
            pixels[i] = u16::from_le_bytes([chunk[0], chunk[1]]);
        }

        if pixels.len() < frame_size {
            return Err(Nd2Error::InvalidFormat(format!(
                "Frame {}: pixel count {} < expected {}",
                index, pixels.len(), frame_size
            )));
        }

        let mut out = vec![0u16; frame_size];
        for y in 0..h {
            for x in 0..w {
                for c in 0..n_c {
                    for comp in 0..n_comp {
                        let src_idx = (y * w * n_c * n_comp) + (x * n_c * n_comp) + (c * n_comp) + comp;
                        let dst_idx = (c * n_comp + comp) * (h * w) + y * w + x;
                        out[dst_idx] = pixels[src_idx];
                    }
                }
            }
        }

        Ok(out)
    }

    fn read_version<R: Read + Seek>(reader: &mut R) -> Result<(u32, u32)> {
        reader.seek(SeekFrom::Start(0))?;

        let mut header = [0u8; 112]; // 4 + 4 + 8 + 32 + 64
        reader.read_exact(&mut header).map_err(|e| {
            Nd2Error::InvalidFormat(format!(
                "Failed to read file header (expected 112 bytes): {}",
                e
            ))
        })?;

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
