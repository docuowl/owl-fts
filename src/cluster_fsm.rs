use crate::buffer::Buffer;
use crate::cluster_fsm::State::PageArrayItem;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
enum State {
    WordLength,
    ClusterLength,
    Word,
    PageSize,
    PageArrayItem,
    PageArrayFrequency,
}

pub struct ClusterFSM {
    state: State,
    word_length: usize,
    cluster_length: usize,
    current_word: String,
    word_buf: Vec<u8>,
    pages_size: usize,
    pages: Vec<u16>,
    frequencies: Vec<u16>,
    page_buf: Buffer,
    clusters: HashMap<String, Vec<Vec<u16>>>,
    all_clusters: HashMap<String, Vec<Vec<u16>>>,
}

impl ClusterFSM {
    pub fn new() -> ClusterFSM {
        ClusterFSM {
            state: State::WordLength,
            word_length: 0,
            cluster_length: 0,
            current_word: "".to_string(),
            word_buf: vec![],
            pages_size: 0,
            pages: vec![],
            frequencies: vec![],
            page_buf: Buffer::new(),
            clusters: HashMap::new(),
            all_clusters: HashMap::new(),
        }
    }

    pub fn feed_buffer(
        &mut self,
        buf: &mut Buffer,
    ) -> Result<HashMap<String, Vec<Vec<u16>>>, Error> {
        loop {
            if buf.remaining_size() == 0 {
                break;
            }
            self.feed(buf.next())?;
        }
        Ok(self.all_clusters.to_owned())
    }

    fn feed(&mut self, b: u8) -> Result<(), Error> {
        match self.state {
            State::WordLength => {
                self.word_length = b as usize;
                self.state = State::ClusterLength;
            }
            State::ClusterLength => {
                self.cluster_length = b as usize;
                self.state = State::Word;
            }
            State::Word => {
                self.word_buf.push(b);
                if self.word_buf.len() == self.word_length {
                    self.current_word = String::from_utf8(Vec::from(&self.word_buf[..]))
                        .map_err(|_| Error::from(ErrorKind::InvalidInput))?;
                    self.word_buf.clear();
                    self.state = State::PageSize;
                }
            }
            State::PageSize => {
                self.pages_size = (b as usize) * 2;
                self.state = State::PageArrayItem;
            }
            State::PageArrayItem => {
                self.page_buf.push(b);
                if self.page_buf.len() == 2 {
                    self.pages.push(self.page_buf.read_u16());
                    self.page_buf.reset();
                    self.state = State::PageArrayFrequency;
                }
            }
            State::PageArrayFrequency => {
                self.page_buf.push(b);
                if self.page_buf.len() == 2 {
                    self.frequencies.push(self.page_buf.read_u16());
                    self.page_buf.reset();
                    self.state = PageArrayItem;
                    if self.pages.len() == self.pages_size / 2 {
                        self.state = State::Word;
                        self.push_word();
                        if self.clusters.len() == self.cluster_length {
                            // FIXME: Not sure how to handle this without cloning...
                            self.all_clusters.extend(self.clusters.to_owned());

                            self.clusters.clear();
                            self.state = State::WordLength;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn push_word(&mut self) {
        self.clusters.insert(
            String::from(&self.current_word),
            vec![Vec::from(&self.pages[..]), Vec::from(&self.frequencies[..])],
        );
        self.current_word = "".to_string();
        self.pages.clear();
        self.frequencies.clear();
    }
}
