use std::future::Future;
use std::pin::Pin;
use std::marker::PhantomData;
use crate::types::error::BirdeyeError;

#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl PaginationParams {
    pub fn new(offset: Option<u32>, limit: Option<u32>) -> Self {
        Self { offset, limit }
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            offset: None,
            limit: Some(10),
        }
    }
}

pub trait PaginatedRequest<T>: Send + Sync {
    fn execute(&self, offset: u32, limit: u32) -> Pin<Box<dyn Future<Output = Result<Vec<T>, BirdeyeError>> + Send>>;
}

impl<F, Fut, T> PaginatedRequest<T> for F
where
    F: Fn(u32, u32) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Vec<T>, BirdeyeError>> + Send + 'static,
{
    fn execute(&self, offset: u32, limit: u32) -> Pin<Box<dyn Future<Output = Result<Vec<T>, BirdeyeError>> + Send>> {
        Box::pin(self(offset, limit))
    }
}

pub struct PaginatedIterator<T, R>
where
    T: Send,
    R: PaginatedRequest<T>,
{
    request: R,
    page_size: u32,
    current_offset: u32,
    total_items: Option<u32>,
    _phantom: PhantomData<T>,
}

impl<T, R> PaginatedIterator<T, R>
where
    T: Send,
    R: PaginatedRequest<T>,
{
    pub fn new(request: R, page_size: u32) -> Self {
        Self {
            request,
            page_size,
            current_offset: 0,
            total_items: None,
            _phantom: PhantomData,
        }
    }

    pub async fn next_page(&mut self) -> Option<Result<Vec<T>, BirdeyeError>> {
        if let Some(total) = self.total_items {
            if self.current_offset >= total {
                return None;
            }
        }

        let result = self.request.execute(self.current_offset, self.page_size).await;
        match result {
            Ok(items) => {
                if items.is_empty() {
                    None
                } else {
                    self.current_offset += items.len() as u32;
                    Some(Ok(items))
                }
            }
            Err(e) => Some(Err(e)),
        }
    }

    pub async fn collect_all(mut self) -> Result<Vec<T>, BirdeyeError> {
        let mut all_items = Vec::new();

        while let Some(result) = self.next_page().await {
            all_items.extend(result?);
        }

        Ok(all_items)
    }
}
