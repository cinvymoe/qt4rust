// Data Source Abstraction

use crate::error::DataResult;

/// DataSource trait - data source abstraction
pub trait DataSource<T> {
    fn read(&self) -> DataResult<T>;
    fn write(&mut self, data: &T) -> DataResult<()>;
    fn is_available(&self) -> bool;
    fn name(&self) -> &str;
}

/// MemoryDataSource - in-memory data source
pub struct MemoryDataSource<T> {
    data: Option<T>,
}

impl<T> MemoryDataSource<T> {
    pub fn new() -> Self {
        Self { data: None }
    }
}

impl<T: Clone> DataSource<T> for MemoryDataSource<T> {
    fn read(&self) -> DataResult<T> {
        self.data
            .clone()
            .ok_or_else(|| crate::error::DataError::NotFound("No data available".to_string()))
    }

    fn write(&mut self, data: &T) -> DataResult<()> {
        self.data = Some(data.clone());
        Ok(())
    }

    fn is_available(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "MemoryDataSource"
    }
}

impl<T> Default for MemoryDataSource<T> {
    fn default() -> Self {
        Self::new()
    }
}
