use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnError, DefaultOnNull};
pub use tantivy::store::Compressor;
use tantivy::{schema::*, Index};
use tantivy_meta_tokenizer::{get_tokenizer, META_TOKENIZER};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_directory::WebDirectory;

pub mod search;
pub mod web_directory;

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Book {
    pub id: u64,

    pub title: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub author: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub publisher: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub extension: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub filesize: u64,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub language: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub year: u64,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub pages: u64,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub isbn: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub ipfs_cid: String,
}

impl From<(&Schema, Document)> for Book {
    fn from((schema, doc): (&Schema, Document)) -> Self {
        macro_rules! get_field_text {
            ($field:expr) => {
                doc.get_first(schema.get_field($field).unwrap())
                    .unwrap()
                    .as_text()
                    .unwrap_or_default()
                    .to_owned()
            };
        }

        macro_rules! get_field_u64 {
            ($field:expr) => {
                doc.get_first(schema.get_field($field).unwrap())
                    .unwrap()
                    .as_u64()
                    .unwrap_or_default()
            };
        }

        Book {
            id: get_field_u64!("id"),
            title: get_field_text!("title"),
            author: get_field_text!("author"),
            publisher: get_field_text!("publisher"),
            extension: get_field_text!("extension"),
            filesize: get_field_u64!("filesize"),
            language: get_field_text!("language"),
            year: get_field_u64!("year"),
            pages: get_field_u64!("pages"),
            isbn: get_field_text!("isbn"),
            ipfs_cid: get_field_text!("ipfs_cid"),
        }
    }
}

#[derive(Clone)]
pub struct Searcher {
    pub compressor: Compressor,

    index: Index,
    schema: Schema,

    // fields
    title: Field,
    author: Field,
    publisher: Field,
    extension: Field,
    language: Field,
    isbn: Field,
}

impl Searcher {
    pub fn new() -> Self {
        let text_indexing = TextFieldIndexing::default()
            .set_tokenizer(META_TOKENIZER)
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        let text_options = TextOptions::default()
            .set_indexing_options(text_indexing)
            .set_stored();

        let mut schema_builder = Schema::builder();
        let _id = schema_builder.add_u64_field("id", INDEXED | STORED);
        let title = schema_builder.add_text_field("title", text_options.clone());
        let author = schema_builder.add_text_field("author", text_options.clone());
        let publisher = schema_builder.add_text_field("publisher", text_options);
        let extension = schema_builder.add_text_field("extension", STRING | STORED);
        let _filesize = schema_builder.add_u64_field("filesize", STORED);
        let language = schema_builder.add_text_field("language", TEXT | STORED);
        let _year = schema_builder.add_u64_field("year", STORED);
        let _pages = schema_builder.add_u64_field("pages", STORED);
        let isbn = schema_builder.add_text_field("isbn", TEXT | STORED);
        let _ipfs_cid = schema_builder.add_text_field("ipfs_cid", STORED);
        let schema = schema_builder.build();

        let directory = WebDirectory::open("http://127.0.0.1:8080/");
        let mut index = Index::open(directory).unwrap();

        index.tokenizers().register(META_TOKENIZER, get_tokenizer());
        _ = index.set_default_multithread_executor();

        Self {
            compressor: Compressor::Lz4,

            index,
            schema,

            title,
            author,
            publisher,
            extension,
            language,
            isbn,
        }
    }
}

#[wasm_bindgen]
pub fn search(query: String) -> JsValue {
    set_panic_hook();
    let searcher = Searcher::new();
    let books = searcher.search(&query, 100);
    serde_wasm_bindgen::to_value(&books).unwrap()
}

pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}
