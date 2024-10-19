use tokio::task::JoinHandle;

pub struct TokioSpawner;

impl TokioSpawner {
    pub fn spawn<F, T>(task: F) -> JoinHandle<T> 
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static
    {
        tokio::spawn(task)
    }
}