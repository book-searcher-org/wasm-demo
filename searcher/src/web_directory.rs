use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use std::{io, ops::Range, path::Path, sync::Arc, usize};
use tantivy::{
    directory::{error::OpenReadError, FileHandle, OwnedBytes},
    Directory, HasLen,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[derive(Clone, Default)]
pub struct WebDirectory {
    base_url: String,
}

impl fmt::Debug for WebDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WebDirectory")
    }
}

impl WebDirectory {
    pub fn open(base_url: &str) -> WebDirectory {
        WebDirectory {
            base_url: base_url.to_string(),
        }
    }

    pub fn get_network_file_handle(&self, file_name: &str) -> WebFile {
        WebFile::new(self.base_url.to_string() + file_name)
    }
}

impl Directory for WebDirectory {
    fn get_file_handle(&self, file_name: &Path) -> Result<Arc<dyn FileHandle>, OpenReadError> {
        Ok(Arc::new(
            self.get_network_file_handle(&file_name.to_string_lossy()),
        ))
    }

    fn exists(&self, path: &Path) -> Result<bool, OpenReadError> {
        Ok(self.get_file_handle(path)?.len() > 0)
    }

    fn atomic_read(&self, path: &Path) -> Result<Vec<u8>, OpenReadError> {
        let file_handle = self.get_network_file_handle(&path.to_string_lossy());
        Ok(file_handle
            .do_read_bytes(None)
            .map_err(|e| OpenReadError::wrap_io_error(e, path.to_path_buf()))?
            .to_vec())
    }

    fn atomic_write(&self, _: &Path, _: &[u8]) -> std::io::Result<()> {
        Ok(())
    }

    fn delete(&self, _: &Path) -> Result<(), tantivy::directory::error::DeleteError> {
        Ok(())
    }

    fn open_write(
        &self,
        _: &Path,
    ) -> Result<tantivy::directory::WritePtr, tantivy::directory::error::OpenWriteError> {
        Ok(std::io::BufWriter::new(Box::new(Noop {})))
    }

    fn sync_directory(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn watch(
        &self,
        _: tantivy::directory::WatchCallback,
    ) -> tantivy::Result<tantivy::directory::WatchHandle> {
        Ok(tantivy::directory::WatchHandle::empty())
    }

    fn acquire_lock(
        &self,
        _lock: &tantivy::directory::Lock,
    ) -> Result<tantivy::directory::DirectoryLock, tantivy::directory::error::LockError> {
        Ok(tantivy::directory::DirectoryLock::from(Box::new(|| {})))
    }
}

#[derive(Debug, Clone)]
pub struct WebFile {
    file_url: String,
}

impl WebFile {
    pub fn new(file_url: String) -> WebFile {
        WebFile { file_url }
    }

    fn do_read_bytes(&self, byte_range: Option<Range<usize>>) -> io::Result<OwnedBytes> {
        let request_response = generate_range_request(&self.file_url, byte_range).request()?;
        Ok(OwnedBytes::new(request_response.data))
    }

    pub fn internal_length(&self) -> Option<u64> {
        let Ok(response )= generate_length_request(&self.file_url).request() else {
            return None
        };
        response.headers.iter().find_map(|header| {
            if header.name == "content-length" {
                Some(header.value.parse::<u64>().unwrap())
            } else {
                None
            }
        })
    }
}

impl FileHandle for WebFile {
    fn read_bytes(&self, byte_range: Range<usize>) -> io::Result<OwnedBytes> {
        self.do_read_bytes(Some(byte_range))
    }
}

impl HasLen for WebFile {
    fn len(&self) -> usize {
        self.internal_length().unwrap() as usize
    }
}

struct Noop {}

impl std::io::Write for Noop {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl tantivy::directory::TerminatingWrite for Noop {
    fn terminate_ref(&mut self, _: tantivy::directory::AntiCallToken) -> std::io::Result<()> {
        Ok(())
    }
}

fn generate_range_request(file_url: &str, range: Option<Range<usize>>) -> WebRequest {
    let mut headers = Vec::new();
    if let Some(range) = range {
        headers.push(Header::new(
            "Range",
            &format!("bytes={}-{}", range.start, range.end - 1),
        ))
    }

    WebRequest {
        method: "GET".to_string(),
        url: file_url.to_string(),
        headers,
    }
}

fn generate_length_request(file_url: &str) -> WebRequest {
    WebRequest {
        method: "HEAD".to_string(),
        url: file_url.to_string(),
        headers: vec![],
    }
}

pub struct WebRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<Header>,
}

impl WebRequest {
    pub fn request(self) -> io::Result<WebResponse> {
        let response = request(
            self.method,
            self.url,
            serde_wasm_bindgen::to_value(&self.headers).unwrap(),
        )
        .unwrap();
        let response: WebResponse = serde_wasm_bindgen::from_value(response).unwrap();
        Ok(response)
    }
}

#[wasm_bindgen(raw_module = "/searcher/lib/request.ts")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub fn request(method: String, url: String, headers: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Header {
    pub fn new(name: &str, value: &str) -> Header {
        Header {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct WebResponse {
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub headers: Vec<Header>,
}
