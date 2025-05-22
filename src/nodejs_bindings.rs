use napi::{Error, Result};
use napi_derive::napi;

#[napi]
fn from_html(html_text: String) -> String {
    crate::from_html(html_text.as_str())
}

#[napi]
fn from_file(html_filepath: String) -> Result<String> {
    match crate::from_file(html_filepath.as_str()) {
        Ok(ret) => Ok(ret),
        Err(err) => Err(Error::from_reason(err)),
    }
}

#[napi]
fn from_html_to_file(html_text: String, markdown_filepath: String, overwrites: bool) -> Result<()> {
    match crate::from_html_to_file(html_text.as_str(), markdown_filepath.as_str(), overwrites) {
        Ok(ret) => Ok(ret),
        Err(err) => Err(Error::from_reason(err)),
    }
}

#[napi]
fn from_file_to_file(
    html_filepath: String,
    markdown_filepath: String,
    overwrites: bool,
) -> Result<()> {
    match crate::from_file_to_file(
        html_filepath.as_str(),
        markdown_filepath.as_str(),
        overwrites,
    ) {
        Ok(ret) => Ok(ret),
        Err(err) => Err(Error::from_reason(err)),
    }
}
