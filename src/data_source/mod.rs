pub mod json;
pub mod file;

pub trait FetchData {
    fn data(&self) -> String;
}
