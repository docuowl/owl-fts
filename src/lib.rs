mod cluster_fsm;
mod buffer;

use crate::buffer::Buffer;
use crate::cluster_fsm::ClusterFSM;
use std::collections::HashMap;
use std::fmt;
use std::io::{Error, ErrorKind, Read};
use flate2::read::GzDecoder;

type PageIndex = usize;
type Frequency = usize;
type Word = String;

#[derive(Debug)]
pub struct SearchResult {
    pub page_id: String,
    pub page_index: PageIndex,
    pub score: f32,
}

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SearchResult {{ score: {}, index: {}, id: {} }}",
            self.score, self.page_index, self.page_id
        )
    }
}

#[derive(Debug)]
pub struct FTS {
    // Clusters = Word to Pages
    clusters: HashMap<Word, Vec<PageIndex>>,
    // Frequencies = Word to Page->Frequency
    frequencies: HashMap<Word, HashMap<PageIndex, Frequency>>,
    pages: Vec<String>,
}

impl FTS {
    pub fn new(input: &str) -> Result<FTS, Error> {
        let raw_bytes = match base64::decode(input) {
            Ok(u) => u,
            Err(_) => {
                return Err(Error::from(ErrorKind::InvalidData));
            }
        };

        let buffer = &mut Buffer::from(raw_bytes);
        if !has_valid_header(buffer) {
            return Err(Error::from(ErrorKind::InvalidData));
        }
        let gzip_size = buffer.read_u32() as usize;
        let gzip_stream = match buffer.read_subbuf(gzip_size) {
            Some(buf) => buf,
            None => return Err(Error::from(ErrorKind::UnexpectedEof)),
        };
        let data_buffer = &mut match read_compressed_stream(gzip_stream) {
            Ok(v) => v,
            Err(err) => return Err(err),
        };
        let pages = match read_section_names(data_buffer) {
            Ok(v) => v,
            Err(err) => return Err(err),
        };
        let mut fsm = ClusterFSM::new();
        let final_result = match fsm.feed_buffer(data_buffer) {
            Err(_) => return Err(Error::from(ErrorKind::InvalidData)),
            Ok(v) => v,
        };

        let mut clusters: HashMap<Word, Vec<PageIndex>> = HashMap::new();
        let mut frequencies: HashMap<Word, HashMap<PageIndex, Frequency>> = HashMap::new();

        for (word, meta) in final_result {
            let page_idx = &meta[0];
            let frequency = &meta[1];
            clusters.insert(
                String::from(&word),
                page_idx.iter().map(|v| *v as usize).collect(),
            );

            for i in 0..page_idx.len() {
                if frequencies.contains_key(&word) {
                    let mut word_freq = frequencies[&word].to_owned();
                    word_freq.insert(page_idx[i] as usize, frequency[i] as usize);
                    frequencies.insert(String::from(&word), word_freq);
                } else {
                    frequencies.insert(
                        String::from(&word),
                        vec![(page_idx[i] as usize, frequency[i] as usize)]
                            .into_iter()
                            .collect(),
                    );
                }
            }
        }

        Ok(FTS {
            clusters,
            frequencies,
            pages,
        })
    }

    pub fn search(&self, input: &str) -> Vec<SearchResult> {
        let input = input.to_lowercase();
        let terms = input.split(' ').into_iter().collect::<Vec<&str>>();
        let mut pages: HashMap<PageIndex, Frequency> = HashMap::new();

        // Locate pages
        for term in &terms {
            let term_pages = self
                .clusters
                .iter()
                .filter(|(word, _)| *word == term)
                .map(|(_, p)| Vec::from(&p[..]))
                .flatten()
                .map(|v| (v, 0))
                .collect::<Vec<(PageIndex, Frequency)>>();
            pages.extend(term_pages);
        }

        // Calculate score for each one
        for term in terms {
            let mut pages_updated = HashMap::new();
            for (pid, score) in &pages {
                if let Some(data) = self.frequencies.get(term) {
                    if let Some(freq) = data.get(&pid) {
                        pages_updated.insert(*pid, *score + *freq);
                    }
                }
            }
            pages.extend(pages_updated);
        }

        if pages.is_empty() {
            return vec![];
        }

        let mut results = pages
            .into_iter()
            .map(|(k, v)| vec![k, v])
            .collect::<Vec<Vec<usize>>>();
        results.sort_by(|a, b| b[1].cmp(&a[1]));
        let max = results.iter().max_by_key(|i| i[1]).map(|v| v[1]).unwrap();
        results
            .into_iter()
            .map(|v| SearchResult {
                page_index: v[0],
                score: v[1] as f32 / max as f32,
                page_id: String::from(self.pages.get(v[0]).unwrap_or(&"[unknown]".to_string())),
            })
            .collect::<Vec<SearchResult>>()
    }
}

fn has_valid_header(buf: &mut Buffer) -> bool {
    let expected_bytes: [u8; 5] = [0x6F, 0x77, 0x6C, 0x00, 0x01];
    if buf.len() <= expected_bytes.len() {
        return false;
    }

    for byte in &expected_bytes {
        if buf.next() != *byte {
            return false;
        }
    }
    true
}

fn read_compressed_stream(buf: Buffer) -> Result<Buffer, Error> {
    let buf = &buf.into_vec()[..];
    let mut decoder = GzDecoder::new(buf);
    let mut inflated = vec![];
    decoder.read_to_end(&mut inflated)?;
    Ok(Buffer::from(inflated))
}

fn read_section_names(buf: &mut Buffer) -> Result<Vec<String>, Error> {
    if buf.next() != 0x02u8 {
        return Err(Error::from(ErrorKind::InvalidData));
    }

    let mut tmp_buf = Vec::with_capacity(512);
    let mut return_value = Vec::new();

    loop {
        if buf.remaining_size() == 0 {
            // EOF. Something's awry.
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }

        match buf.next() {
            0x00 => {
                let str = String::from_utf8(Vec::from(&tmp_buf[..]))
                    .map_err(|_| Error::from(ErrorKind::InvalidInput))?;
                return_value.push(str);
                tmp_buf.clear();
                continue;
            }
            0x03 => break,
            byte => tmp_buf.push(byte),
        }
    }

    Ok(return_value)
}
