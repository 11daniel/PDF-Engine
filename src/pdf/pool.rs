use std::{
    pin::pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread::{self, sleep},
    time::Duration,
};

use tokio::sync::{mpsc, oneshot};

use crate::error::BoxedError;

use super::PdfGenerateOptions;

pub struct PdfGenerateRequest {
    pub callback: oneshot::Sender<Vec<u8>>,
    pub options: Arc<PdfGenerateOptions>,
}

#[derive(Debug)]
pub struct PdfPool {
    senders: Arc<Vec<mpsc::Sender<PdfGenerateRequest>>>,
    next: AtomicUsize,
}

fn pdf_render_handler(mut receiver: mpsc::Receiver<PdfGenerateRequest>, thread_id: usize) {
    log::debug!("PDF worker thread {thread_id} started");

    while let Some(request) = receiver.blocking_recv() {
        sleep(Duration::from_secs_f64(0.020));

        let _ = request.callback.send(vec![]);
    }
}

impl PdfPool {
    pub fn new(thread_count: usize) -> Self {
        let mut senders = Vec::with_capacity(thread_count);

        for i in 0..thread_count {
            let (tx, rx) = mpsc::channel(32);
            let handle = thread::spawn(move || {
                println!("Worker thread {i} started");
                pdf_render_handler(rx, i);
            });
            // we keep handle pinned otherwise the thread might get dropped
            pin!(handle);
            senders.push(tx);
        }

        Self {
            senders: Arc::new(senders),
            next: AtomicUsize::new(0),
        }
    }

    pub async fn render_pdf(
        &self,
        options: Arc<PdfGenerateOptions>,
    ) -> Result<Vec<u8>, BoxedError> {
        let index = self.next.fetch_add(1, Ordering::Relaxed) % self.senders.len();
        let tx = &self.senders[index];

        let (res_tx, res_rx) = oneshot::channel();
        tx.send(PdfGenerateRequest {
            callback: res_tx,
            options,
        })
        .await?;

        Ok(res_rx.await?)
    }
}
