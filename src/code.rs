use bytes::Bytes;
use lazy_static::lazy_static;
use nom_locate::LocatedSpan;
use sha2::{Digest, Sha256};
use std::hash::Hash;
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::Path,
    sync::{Arc, Mutex},
};

lazy_static! {
    /// Global interner keyed by SHA‑256 so every identical buffer shares one Arc
    pub static ref CODE_ATLAS: Arc<Mutex<HashMap<Bytes, Arc<Code>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// In‑memory source buffer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code {
    sha: Bytes,
    pub name: Option<String>,
    pub file_path: Option<String>,
    pub text: Arc<String>,
}

impl Hash for Code {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.sha.hash(state);
    }
}

impl Code {
    /// Load file, compute SHA, return *interned* Arc<Code>
    pub fn from_file(path: &Path) -> std::io::Result<Arc<Self>> {
        let mut buf = String::new();
        File::open(path)?.read_to_string(&mut buf)?;
        Ok(Self::intern(buf, Some(path.to_string_lossy().into())))
    }

    /// Create from raw snippet (REPL, tests, etc.)
    pub fn from_snippet(src: &str) -> Arc<Self> {
        Self::intern(src.to_owned(), None)
    }

    /// Borrow `&'a str` and build the initial `ParserSpan`
    pub fn span(arc: &Arc<Self>) -> ParserSpan {
        ParserSpan::new_extra(arc.text.as_str(), arc.clone())
    }

    fn intern(text: String, file_path: Option<String>) -> Arc<Self> {
        let sha = Bytes::from(Sha256::digest(text.as_bytes()).to_vec());
        let mut atlas = CODE_ATLAS.lock().unwrap();
        atlas
            .entry(sha.clone())
            .or_insert_with(|| {
                Arc::new(Code {
                    sha,
                    name: file_path
                        .as_ref()
                        .map(|p| Path::new(p).file_name().unwrap().to_string_lossy().into()),
                    file_path,
                    text: Arc::new(text),
                })
            })
            .clone()
    }
}

pub type ParserSpan<'a> = LocatedSpan<&'a str, Arc<Code>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeSpan {
    pub code: Arc<Code>,
    pub start: usize,
    pub end: usize,
}

impl CodeSpan {
    pub fn new(code: Arc<Code>, start: usize, end: usize) -> Self {
        Self { code, start, end }
    }
}

impl<'a> From<ParserSpan<'a>> for CodeSpan {
    fn from(s: ParserSpan<'a>) -> Self {
        let start = s.location_offset();
        let end = start + s.fragment().len();
        CodeSpan::new(s.extra.clone(), start, end)
    }
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub value: T,
    pub span: CodeSpan,
}
