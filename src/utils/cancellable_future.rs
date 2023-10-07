use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;
use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};

#[pin_project]
pub struct CancellableFuture<'a, F, T>
where
    F: Future<Output = T>,
{
    #[pin]
    first_future: F,
    #[pin]
    cancel_future: WaitForCancellationFuture<'a>,
}

impl<'a, F, T> CancellableFuture<'a, F, T>
where
    F: Future<Output = T>,
{
    pub fn new(first_future: F, cancel_token: &'a CancellationToken) -> Self {
        CancellableFuture {
            first_future,
            cancel_future: cancel_token.cancelled(),
        }
    }
}

impl<'a, F, T> Future for CancellableFuture<'a, F, T>
where
    F: Future<Output = T>,
{
    type Output = Result<T, ()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.first_future.poll(cx) {
            Poll::Ready(x) => Poll::Ready(Ok(x)),
            Poll::Pending => match this.cancel_future.poll(cx) {
                Poll::Ready(_) => Poll::Ready(Err(())),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
