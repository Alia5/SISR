use std::{
    fs::File,
    io::Write,
    path::Path,
    sync::{Arc, Mutex, OnceLock, RwLock},
};

use tracing::{Level, Subscriber, level_filters::LevelFilter};
use tracing_subscriber::{
    Layer, Registry,
    filter::Targets,
    fmt,
    layer::{Context, SubscriberExt},
    reload,
};

type BoxedSink = Box<dyn LogSink>;
type SinkList = Arc<RwLock<Vec<BoxedSink>>>;

static STDERR_LEVEL_HANDLE: OnceLock<reload::Handle<LevelFilter, Registry>> = OnceLock::new();
static SINK_LIST: OnceLock<SinkList> = OnceLock::new();

pub fn setup() {
    let initial_level = if cfg!(debug_assertions) {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    let targets = Targets::new()
        .with_default(LevelFilter::TRACE)
        .with_target("calloop", LevelFilter::DEBUG);

    let (stderr_filter, stderr_handle) = reload::Layer::new(initial_level);
    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(targets.clone());

    let sinks: SinkList = Arc::new(RwLock::new(Vec::new()));
    let buffer = Arc::new(Mutex::new(Vec::new()));

    let fmt_layer = fmt::layer()
        .with_writer(MultiSinkBuffer {
            buf: buffer.clone(),
        })
        .with_ansi(false);

    let sink_layer = MultiSinkLayer {
        sinks: sinks.clone(),
        fmt_layer,
        buffer,
    }
    .with_filter(targets);

    let subscriber = Registry::default()
        .with(stderr_filter)
        .with(stderr_layer)
        .with(sink_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

    STDERR_LEVEL_HANDLE.set(stderr_handle).ok();
    SINK_LIST.set(sinks).ok();
}

pub fn set_level(level: Level) {
    if let Some(handle) = STDERR_LEVEL_HANDLE.get() {
        handle
            .reload(LevelFilter::from_level(level))
            .expect("Failed to reload stderr level");
    }
}

pub fn add_sink(sink: impl LogSink + 'static) {
    SINK_LIST
        .get()
        .unwrap()
        .write()
        .unwrap()
        .push(Box::new(sink));
}

pub fn add_file(path: &Path, level: Level) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create log directory");
    }

    let file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .expect("Failed to open log file");

    add_sink(FileSink {
        file: Mutex::new(file),
        level: LevelFilter::from_level(level),
    });
}

pub trait LogSink: Send + Sync {
    fn write(&self, formatted: &str);
    fn level_filter(&self) -> LevelFilter;
}

struct MultiSinkBuffer {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for MultiSinkBuffer {
    type Writer = Self;
    fn make_writer(&'a self) -> Self::Writer {
        self.buf.lock().unwrap().clear();
        Self {
            buf: self.buf.clone(),
        }
    }
}

impl Write for MultiSinkBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct MultiSinkLayer<L> {
    sinks: SinkList,
    fmt_layer: L,
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl<S, L> Layer<S> for MultiSinkLayer<L>
where
    S: Subscriber,
    L: Layer<S>,
{
    fn on_event(&self, event: &tracing::Event, ctx: Context<'_, S>) {
        let sinks = match self.sinks.read() {
            Ok(s) => s,
            Err(_) => return,
        };

        if sinks.is_empty() {
            return;
        }

        let metadata = event.metadata();
        let event_level = *metadata.level();

        self.fmt_layer.on_event(event, ctx.clone());

        let formatted = {
            let buf = self.buffer.lock().unwrap();
            String::from_utf8_lossy(&buf).to_string()
        }
        .trim()
        .to_string();

        for sink in sinks.iter() {
            if sink.level_filter() >= event_level {
                sink.write(&formatted);
            }
        }
    }
}

struct FileSink {
    file: Mutex<File>,
    level: LevelFilter,
}

impl LogSink for FileSink {
    fn write(&self, formatted: &str) {
        if let Ok(mut f) = self.file.lock() {
            let _ = writeln!(f, "{}", formatted);
        }
    }

    fn level_filter(&self) -> LevelFilter {
        self.level
    }
}
