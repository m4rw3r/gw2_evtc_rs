
use Metadata;

use serde::ser::Serialize;
use serde::ser::Serializer;

pub trait Entry: Serialize {
    fn new(u64) -> Self;
    fn time(&self) -> u64;
}

pub struct Series<E: Entry> {
    current: Option<E>,
    series:  Vec<E>,
}

impl<E: Entry> Series<E> {
    pub fn new(meta: &Metadata) -> Self {
        assert!(meta.log_end() > meta.log_start());

        Series {
            current: None,
            series:  Vec::with_capacity(((meta.log_end() - meta.log_start()) / 1000) as usize),
        }
    }

    #[inline]
    pub fn current(&mut self, time: u64) -> &mut E {
        if self.current.is_some() && time != self.current.as_ref().map(Entry::time).unwrap_or(0) {
            self.series.push(self.current.take().unwrap());
        }

        self.current.get_or_insert(Entry::new(time))
    }

    pub fn finalize(&mut self) {
        if let Some(e) = self.current.take() {
            self.series.push(e);
        }
    }
}

impl<E: Entry> Serialize for Series<E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where S: Serializer {
        self.series.serialize(serializer)
    }
}
