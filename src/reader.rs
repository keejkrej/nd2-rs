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
            return Err(Nd2Error::unsupported_version(version.0, version.1));
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
                        _ => clx.clone(),
                    }
                } else {
                    clx.clone()
                };
                let mut exp = parse_experiment(to_parse).unwrap_or_default();
                // If unwrapped gave empty, try parsing root directly (some v3 files differ)
                if exp.is_empty() && self.version.0 >= 3 {
                    exp = parse_experiment(clx).unwrap_or_default();
                }
                self.experiment = Some(exp);
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
        let width = attrs
            .width_px
            .or(attrs.width_bytes.map(|w| {
                let bpp = attrs.bits_per_component_in_memory / 8;
                w / (bpp * attrs.component_count)
            }))
            .unwrap_or(0) as usize;

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

    /// Loop indices for each sequence chunk: seq_index -> axis name -> index.
    /// Channel is omitted when stored in-pixel instead of as separate chunks.
    pub fn loop_indices(&mut self) -> Result<Vec<HashMap<String, usize>>> {
        let (axis_order, coord_shape) = self.coord_axis_order()?;
        let total: usize = coord_shape.iter().product();

        let mut out = Vec::with_capacity(total);
        let n = axis_order.len();

        for seq in 0..total {
            let mut idx = seq;
            let mut m = HashMap::new();
            // Unravel seq: innermost acquisition axis varies fastest
            for i in (0..n).rev() {
                let coord = idx % coord_shape[i];
                idx /= coord_shape[i];
                m.insert(axis_order[i].to_string(), coord);
            }
            out.push(m);
        }

        Ok(out)
    }

    /// Read one frame by sequence index. Returns pixels as (C, Y, X) u16 data.
    pub fn read_frame(&mut self, index: usize) -> Result<Vec<u16>> {
        let attrs = self.attributes()?.clone();
        let max_seq = attrs.sequence_count as usize;
        let chunk_name = format!("ImageDataSeq|{}!", index);
        let chunk_key = chunk_name.as_bytes();

        let h = attrs.height_px as usize;
        let w = attrs.width_px.unwrap_or(0) as usize;
        let (n_c, n_comp) = match attrs.channel_count {
            Some(ch) if ch > 0 => (ch as usize, (attrs.component_count / ch) as usize),
            _ => (attrs.component_count as usize, 1),
        };
        let bytes_per_pixel = (attrs.bits_per_component_in_memory / 8) as usize;
        if bytes_per_pixel == 0 {
            return Err(Nd2Error::file_invalid_format(
                "Invalid bits_per_component_in_memory".to_string(),
            ));
        }
        let raw_row_bytes = attrs.width_bytes.map(|w| w as usize).unwrap_or_else(|| {
            w.saturating_mul(n_c)
                .saturating_mul(n_comp)
                .saturating_mul(bytes_per_pixel)
        });
        if raw_row_bytes == 0 {
            return Err(Nd2Error::file_invalid_format(
                "Invalid frame row stride".to_string(),
            ));
        }
        if raw_row_bytes % bytes_per_pixel != 0 {
            return Err(Nd2Error::file_invalid_format(format!(
                "Frame row stride {} is not divisible by bytes per pixel {}",
                raw_row_bytes, bytes_per_pixel
            )));
        }
        let raw_row_pixels = raw_row_bytes / bytes_per_pixel;

        let frame_size = h
            .checked_mul(w)
            .and_then(|v| v.checked_mul(n_c))
            .and_then(|v| v.checked_mul(n_comp))
            .ok_or_else(|| {
                Nd2Error::file_invalid_format("Frame dimensions overflow".to_string())
            })?;
        let expected_raw = h
            .checked_mul(raw_row_bytes)
            .ok_or_else(|| Nd2Error::file_invalid_format("Frame byte size overflow".to_string()))?;
        let frame_area = h
            .checked_mul(w)
            .ok_or_else(|| Nd2Error::file_invalid_format("Frame area overflow".to_string()))?;
        let n_c_n_comp = n_c.checked_mul(n_comp).ok_or_else(|| {
            Nd2Error::file_invalid_format("Frame channel/component overflow".to_string())
        })?;
        if raw_row_pixels < n_c_n_comp.saturating_mul(w) {
            return Err(Nd2Error::file_invalid_format(format!(
                "Frame row stride {} pixels is smaller than required width {}",
                raw_row_pixels,
                n_c_n_comp.saturating_mul(w)
            )));
        }

        let pixel_bytes = match attrs.compression_type {
            Some(CompressionType::Lossless) => {
                let data = match self.read_raw_chunk(chunk_key) {
                    Ok(data) => data,
                    Err(err) => {
                        if matches!(
                            err,
                            Nd2Error::File {
                                source: crate::error::FileError::ChunkNotFound { .. },
                            }
                        ) {
                            return Err(Nd2Error::input_out_of_range(
                                "sequence index",
                                index,
                                max_seq,
                            ));
                        }
                        return Err(err);
                    }
                };

                if data.len() < 8 {
                    return Err(Nd2Error::file_invalid_format(format!(
                        "Frame {} compressed chunk too short ({} bytes)",
                        index,
                        data.len()
                    )));
                }
                let mut decoder = ZlibDecoder::new(&data[8..]);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                decompressed
            }
            _ => match self.read_uncompressed_frame_bytes(chunk_key, expected_raw) {
                Ok(data) => data,
                Err(err) => {
                    if matches!(
                        err,
                        Nd2Error::File {
                            source: crate::error::FileError::ChunkNotFound { .. },
                        }
                    ) {
                        return Err(Nd2Error::input_out_of_range(
                            "sequence index",
                            index,
                            max_seq,
                        ));
                    }
                    return Err(err);
                }
            },
        };

        if pixel_bytes.len() % 2 != 0 {
            return Err(Nd2Error::file_invalid_format(format!(
                "Frame {}: pixel data length {} is not divisible by 2",
                index,
                pixel_bytes.len()
            )));
        }

        if pixel_bytes.len() / 2 < frame_size {
            return Err(Nd2Error::file_invalid_format(format!(
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
            return Err(Nd2Error::file_invalid_format(format!(
                "Frame {}: pixel count {} < expected {}",
                index,
                pixels.len(),
                frame_size
            )));
        }

        let mut out = vec![0u16; frame_size];
        let row_pixels = raw_row_pixels;

        for y in 0..h {
            let y_offset = y.checked_mul(row_pixels).ok_or_else(|| {
                Nd2Error::file_invalid_format("Frame offset overflow".to_string())
            })?;
            let y_plane_offset = y.checked_mul(w).ok_or_else(|| {
                Nd2Error::file_invalid_format("Frame plane offset overflow".to_string())
            })?;
            for x in 0..w {
                let x_offset = x.checked_mul(n_c_n_comp).ok_or_else(|| {
                    Nd2Error::file_invalid_format("Frame offset overflow".to_string())
                })?;
                for c in 0..n_c {
                    let c_offset = c.checked_mul(n_comp).ok_or_else(|| {
                        Nd2Error::file_invalid_format("Frame offset overflow".to_string())
                    })?;
                    for comp in 0..n_comp {
                        let src_idx = y_offset
                            .checked_add(x_offset)
                            .and_then(|v| v.checked_add(c_offset))
                            .and_then(|v| v.checked_add(comp))
                            .ok_or_else(|| {
                                Nd2Error::file_invalid_format("Frame offset overflow".to_string())
                            })?;
                        let dst_x = y_plane_offset.checked_add(x).ok_or_else(|| {
                            Nd2Error::file_invalid_format("Frame offset overflow".to_string())
                        })?;
                        let c_plane = c_offset.checked_add(comp).ok_or_else(|| {
                            Nd2Error::file_invalid_format("Frame offset overflow".to_string())
                        })?;
                        let dst_idx = c_plane
                            .checked_mul(frame_area)
                            .and_then(|v| v.checked_add(dst_x))
                            .ok_or_else(|| {
                                Nd2Error::file_invalid_format("Frame offset overflow".to_string())
                            })?;
                        out[dst_idx] = pixels[src_idx];
                    }
                }
            }
        }

        Ok(out)
    }

    fn read_uncompressed_frame_bytes(
        &mut self,
        chunk_key: &[u8],
        expected_raw: usize,
    ) -> Result<Vec<u8>> {
        // ImageDataSeq chunkmap "size" is not a reliable byte count for raw image payloads
        // in all ND2 variants. Read through the chunk header instead and strip the 8-byte
        // frame prefix that precedes the pixel buffer.
        let data = self.read_raw_chunk(chunk_key)?;
        let required = expected_raw
            .checked_add(8)
            .ok_or_else(|| Nd2Error::file_invalid_format("Frame byte size overflow".to_string()))?;

        if data.len() < required {
            return Err(Nd2Error::file_invalid_format(format!(
                "Frame chunk '{}' too small: {} bytes < required {} bytes",
                String::from_utf8_lossy(chunk_key),
                data.len(),
                required
            )));
        }

        Ok(data[8..required].to_vec())
    }

    /// Build axis order and coord shape for seq_index (chunk lookup).
    /// sequence_count = number of ImageDataSeq chunks. When channels are "in-pixel"
    /// (stored within each chunk), sequence_count = product(experiment loops) and we
    /// must NOT include C in axis_order for chunk indexing.
    fn coord_axis_order(&mut self) -> Result<(Vec<&'static str>, Vec<usize>)> {
        let attrs = self.attributes()?.clone();
        let exp = self.experiment()?.clone();
        let n_chan = attrs.channel_count.unwrap_or(attrs.component_count) as usize;
        let seq_count = attrs.sequence_count as usize;

        let mut axis_order: Vec<&'static str> = Vec::new();
        let mut coord_shape: Vec<usize> = Vec::new();

        if exp.is_empty() {
            // Fallback: P,T,C,Z (matches sizes() fallback)
            let n_z = 1;
            let n_pos = 1;
            let n_time = seq_count / (n_pos * n_chan * n_z).max(1);
            axis_order.extend([AXIS_P, AXIS_T, AXIS_C, AXIS_Z]);
            coord_shape.extend([n_pos, n_time, n_chan, n_z]);
        } else {
            // parse_experiment collects nested loop wrappers before the innermost
            // acquisition axes that actually vary fastest in ImageDataSeq numbering.
            for loop_ in exp.iter().rev() {
                match loop_ {
                    crate::types::ExpLoop::TimeLoop(t) => {
                        axis_order.push(AXIS_T);
                        coord_shape.push(t.count as usize);
                    }
                    crate::types::ExpLoop::NETimeLoop(n) => {
                        axis_order.push(AXIS_T);
                        coord_shape.push(n.count as usize);
                    }
                    crate::types::ExpLoop::XYPosLoop(xy) => {
                        axis_order.push(AXIS_P);
                        coord_shape.push(xy.count as usize);
                    }
                    crate::types::ExpLoop::ZStackLoop(z) => {
                        axis_order.push(AXIS_Z);
                        coord_shape.push(z.count as usize);
                    }
                    crate::types::ExpLoop::CustomLoop(_) => {}
                }
            }
            // Add missing axes with size 1 (matching sizes())
            if !axis_order.contains(&AXIS_P) {
                axis_order.push(AXIS_P);
                coord_shape.push(1);
            }
            if !axis_order.contains(&AXIS_T) {
                axis_order.push(AXIS_T);
                coord_shape.push(1);
            }
            if !axis_order.contains(&AXIS_Z) {
                axis_order.push(AXIS_Z);
                coord_shape.push(1);
            }
            // Only add C (and ensure Z) when sequence_count indicates chunks span channel
            let exp_product: usize = coord_shape.iter().product();
            if exp_product > 0 && exp_product * n_chan <= seq_count {
                axis_order.push(AXIS_C);
                coord_shape.push(n_chan);
            }
            if !axis_order.contains(&AXIS_Z) {
                axis_order.push(AXIS_Z);
                coord_shape.push(1);
            }
        }

        Ok((axis_order, coord_shape))
    }

    /// Compute sequence index from (p,t,c,z) using the sequence chunk axis order.
    fn seq_index_from_coords(&mut self, p: usize, t: usize, c: usize, z: usize) -> Result<usize> {
        let (axis_order, coord_shape) = self.coord_axis_order()?;
        let coords: Vec<usize> = axis_order
            .iter()
            .map(|&ax| match ax {
                AXIS_P => p,
                AXIS_T => t,
                AXIS_C => c,
                AXIS_Z => z,
                _ => 0,
            })
            .collect();

        if coords.len() != coord_shape.len() {
            return Err(Nd2Error::file_invalid_format(
                "Coord/axis length mismatch".to_string(),
            ));
        }

        for (idx, (&coord, &shape)) in coords.iter().zip(coord_shape.iter()).enumerate() {
            if shape == 0 {
                return Err(Nd2Error::file_invalid_format(format!(
                    "Invalid axis length: {} has size 0",
                    axis_order[idx]
                )));
            }
            if coord >= shape {
                return Err(Nd2Error::input_out_of_range(
                    format!("axis {}", axis_order[idx]),
                    coord,
                    shape,
                ));
            }
        }

        let mut seq = 0usize;
        let mut stride = 1;
        for i in (0..coords.len()).rev() {
            let next = coords[i]
                .checked_mul(stride)
                .ok_or_else(|| Nd2Error::internal_overflow("sequence index multiply"))?;
            seq = seq
                .checked_add(next)
                .ok_or_else(|| Nd2Error::internal_overflow("sequence index add"))?;
            stride = stride
                .checked_mul(coord_shape[i])
                .ok_or_else(|| Nd2Error::internal_overflow("sequence stride multiply"))?;
        }
        Ok(seq)
    }

    /// Read 2D Y×X frame at (p,t,c,z). Returns the Y×X pixels for the requested channel.
    pub fn read_frame_2d(&mut self, p: usize, t: usize, c: usize, z: usize) -> Result<Vec<u16>> {
        let sizes = self.sizes()?;
        let height = *sizes.get(AXIS_Y).ok_or_else(|| {
            Nd2Error::file_invalid_format("Missing height (Y) dimension".to_string())
        })?;
        let width = *sizes.get(AXIS_X).ok_or_else(|| {
            Nd2Error::file_invalid_format("Missing width (X) dimension".to_string())
        })?;
        let n_pos = *sizes.get(AXIS_P).ok_or_else(|| {
            Nd2Error::file_invalid_format("Missing position (P) dimension".to_string())
        })?;
        let n_time = *sizes.get(AXIS_T).ok_or_else(|| {
            Nd2Error::file_invalid_format("Missing time (T) dimension".to_string())
        })?;
        let n_chan = *sizes.get(AXIS_C).ok_or_else(|| {
            Nd2Error::file_invalid_format("Missing channel (C) dimension".to_string())
        })?;
        let n_z = *sizes
            .get(AXIS_Z)
            .ok_or_else(|| Nd2Error::file_invalid_format("Missing Z dimension".to_string()))?;

        if p >= n_pos {
            return Err(Nd2Error::input_out_of_range("position index", p, n_pos));
        }
        if t >= n_time {
            return Err(Nd2Error::input_out_of_range("time index", t, n_time));
        }
        if c >= n_chan {
            return Err(Nd2Error::input_out_of_range("channel index", c, n_chan));
        }
        if z >= n_z {
            return Err(Nd2Error::input_out_of_range("z index", z, n_z));
        }

        let seq_index = self.seq_index_from_coords(p, t, c, z)?;

        let frame = self.read_frame(seq_index)?;
        let len = height.checked_mul(width).ok_or_else(|| {
            Nd2Error::file_invalid_format("Frame dimensions overflow".to_string())
        })?;

        // Frame is (C,Y,X) planar: channel c is at [c*len..(c+1)*len]
        let start = c.checked_mul(len).ok_or_else(|| {
            Nd2Error::file_invalid_format("Frame slice start overflow".to_string())
        })?;
        let end = (c + 1)
            .checked_mul(len)
            .ok_or_else(|| Nd2Error::file_invalid_format("Frame slice end overflow".to_string()))?;
        if end > frame.len() {
            return Err(Nd2Error::file_invalid_format(format!(
                "Frame data too short for requested channel: frame {} < {}",
                frame.len(),
                end
            )));
        }
        Ok(frame[start..end].to_vec())
    }

    fn read_version<R: Read + Seek>(reader: &mut R) -> Result<(u32, u32)> {
        reader.seek(SeekFrom::Start(0))?;

        let mut header = [0u8; 112]; // 4 + 4 + 8 + 32 + 64
        reader.read_exact(&mut header).map_err(|e| {
            Nd2Error::file_invalid_format(format!(
                "Failed to read file header (expected 112 bytes): {}",
                e
            ))
        })?;

        let magic = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);

        if magic == JP2_MAGIC {
            return Ok((1, 0)); // Legacy format
        }

        if magic != ND2_CHUNK_MAGIC {
            return Err(Nd2Error::file_invalid_magic(ND2_CHUNK_MAGIC, magic));
        }

        let name_length = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        let data_length = u64::from_le_bytes([
            header[8], header[9], header[10], header[11], header[12], header[13], header[14],
            header[15],
        ]);

        // Validate header
        if name_length != 32 || data_length != 64 {
            return Err(Nd2Error::file_invalid_format(
                "Corrupt file header".to_string(),
            ));
        }

        // Check signature
        let name = &header[16..48];
        if name != ND2_FILE_SIGNATURE {
            return Err(Nd2Error::file_invalid_format(
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
