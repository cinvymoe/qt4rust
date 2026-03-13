// Repository Pattern Implementation

use crate::error::DataResult;

/// Repository trait - data repository abstraction
pub trait Repository<T, K = String> {
    fn get(&self, id: &K) -> DataResult<Option<T>>;
    fn get_all(&self) -> DataResult<Vec<T>>;
    fn save(&mut self, data: &T) -> DataResult<()>;
    fn update(&mut self, id: &K, data: &T) -> DataResult<()>;
    fn delete(&mut self, id: &K) -> DataResult<()>;
}

/// HasId trait - for extracting data ID
pub trait HasId<K> {
    fn id(&self) -> &K;
}
