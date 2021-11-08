use crate::stream_op::*;
use std::collections::VecDeque;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[macro_export]

/// `row_nonnull` builds a `Row` with concrete values.
/// TODO: add macro row!, which requires a new trait `ToScalarValue`.
macro_rules! row_nonnull {
  [$( $value:expr ),*] => {
    {
      Row(vec![$(Some($value.to_scalar_value()), )*])
    }
  };
}

pub struct MockSource {
    schema: Schema,
    msgs: VecDeque<Message>,
}

impl MockSource {
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            msgs: VecDeque::default(),
        }
    }

    pub fn with_chunks(schema: Schema, chunks: Vec<StreamChunk>) -> Self {
        Self {
            schema,
            msgs: chunks.into_iter().map(Message::Chunk).collect(),
        }
    }

    pub fn push_chunks(&mut self, chunks: impl Iterator<Item = StreamChunk>) {
        self.msgs.extend(chunks.map(Message::Chunk));
    }

    pub fn push_barrier(&mut self, epoch: u64, stop: bool) {
        self.msgs.push_back(Message::Barrier { epoch, stop });
    }
}

#[async_trait]
impl Executor for MockSource {
    async fn next(&mut self) -> Result<Message> {
        match self.msgs.pop_front() {
            Some(msg) => Ok(msg),
            None => Ok(Message::Terminate),
        }
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }
}

/// This source takes message from users asynchronously
pub struct MockAsyncSource {
    schema: Schema,
    rx: UnboundedReceiver<Message>,
}

impl MockAsyncSource {
    pub fn new(schema: Schema, rx: UnboundedReceiver<Message>) -> Self {
        Self { schema, rx }
    }

    pub fn push_chunks(
        tx: &mut UnboundedSender<Message>,
        chunks: impl IntoIterator<Item = StreamChunk>,
    ) {
        for chunk in chunks.into_iter() {
            tx.send(Message::Chunk(chunk)).expect("Receiver closed");
        }
    }

    pub fn push_barrier(tx: &mut UnboundedSender<Message>, epoch: u64, stop: bool) {
        tx.send(Message::Barrier { epoch, stop })
            .expect("Receiver closed");
    }
}

#[async_trait]
impl Executor for MockAsyncSource {
    async fn next(&mut self) -> Result<Message> {
        match self.rx.recv().await {
            Some(msg) => Ok(msg),
            None => Ok(Message::Terminate),
        }
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }
}
