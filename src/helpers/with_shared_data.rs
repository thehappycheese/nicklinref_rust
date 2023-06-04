use std::convert::Infallible;
use warp::Filter;

/// This function embeds a shared piece of data into a warp filter chain by
/// taking ownership of a clone, then providing a fresh clone each time the
/// filter is executed.
///
/// The shared_data must generally be some object wrapped in a `std::sync::Arc`
/// in order to satisfy `Send + Sync + Clone`.
///
/// # Type Parameters
///
/// * `T` - The type of the shared data. It must be thread-safe (i.e., implement `Send + Sync`) and cloneable (`Clone`).
/// 
/// # Parameters
///
/// * `shared_data` - The shared data to be moved into the filter.
///
/// # Returns
///
/// A filter that, when called, clones the shared data.
///
/// # Example
///
/// ```rust
/// use std::sync::Arc;
/// use warp::Filter;
/// use warp_helpers::with_shared_data;
///
/// let data = Arc::new(vec![1, 2, 3]);
/// let data_filter = with_shared_data(data.clone());
///
/// let route = warp::path!("data" / usize)
///     .and(data_filter.clone())
///     .map(|index, data: Arc<Vec<i32>>| {
///         format!("The data at position {} is {}", index, data[index])
///     });
/// ```
pub fn with_shared_data<T>(shared_data: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone
where
    T: Send + Sync + Clone,
{
    warp::any().map(move || shared_data.clone())
}