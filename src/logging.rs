use std::io;

use log::{Level, LevelFilter, Log, Record, set_boxed_logger, set_max_level};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::cli::Cli;

type FnGetWriter<W> = fn() -> W;

pub struct AppLogger<W: io::Write + WriteColor> {
    level_filter: LevelFilter,
    writer: FnGetWriter<W>,
}

impl<W: io::Write + WriteColor> AppLogger<W> {
    fn new(level_filter: LevelFilter, writer: FnGetWriter<W>) -> AppLogger<W> {
        AppLogger {
            level_filter,
            writer,
        }
    }

    pub fn init(cfg: &Cli) {
        let level_filter = cfg.verbosity.log_level_filter();
        let logger = Box::new(AppLogger::<StandardStream>::new(level_filter, || {
            StandardStream::stderr(ColorChoice::Auto)
        }));
        set_boxed_logger(logger).unwrap();
        set_max_level(level_filter);
    }
}

impl<W: io::Write + WriteColor> Log for AppLogger<W> {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level_filter
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let mut output = (self.writer)();
        format_record(&mut output, record).ok();
    }

    fn flush(&self) {}
}

fn format_record<W: io::Write + WriteColor>(w: &mut W, record: &Record) -> io::Result<()> {
    let mut color = ColorSpec::new();
    match record.level() {
        Level::Error => color.set_fg(Some(Color::Red)).set_bold(true),
        Level::Warn => color.set_fg(Some(Color::Yellow)).set_bold(true),
        Level::Info => color.set_fg(Some(Color::White)),
        Level::Debug => color.set_fg(Some(Color::Cyan)),
        Level::Trace => color.set_fg(Some(Color::Blue)),
    };

    if w.supports_color() {
        w.set_color(&color)?;
    }
    writeln!(w, "{}", record.args())?;
    w.reset()?;
    w.flush()?;

    Ok(())
}

#[cfg(test)]
mod testing {
    use std::sync::{Arc, LazyLock, Mutex};

    use clap_verbosity_flag::Verbosity;
    use log::log_enabled;

    use super::*;

    fn get_error_color() -> ColorSpec {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(Color::Red)).set_bold(true);
        spec
    }
    fn get_warn_color() -> ColorSpec {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(Color::Yellow)).set_bold(true);
        spec
    }
    fn get_info_color() -> ColorSpec {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(Color::White));
        spec
    }
    fn get_debug_color() -> ColorSpec {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(Color::Cyan));
        spec
    }
    fn get_trace_color() -> ColorSpec {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(Color::Blue));
        spec
    }

    struct BufferedStream {
        lines: Vec<Vec<u8>>,
        flushes: u8,

        colors: Vec<ColorSpec>,
        resets: u8,
    }

    impl BufferedStream {
        fn new() -> BufferedStream {
            BufferedStream {
                lines: Vec::new(),
                flushes: 0,
                colors: Vec::new(),
                resets: 0,
            }
        }

        fn restore(&mut self) {
            self.lines.clear();
            self.flushes = 0;
            self.colors.clear();
            self.resets = 0;
        }
    }

    impl io::Write for BufferedStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let data = buf.to_vec();
            let size = data.len();
            self.lines.push(data);
            Ok(size)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }

    impl WriteColor for BufferedStream {
        fn supports_color(&self) -> bool {
            true
        }

        fn set_color(&mut self, spec: &ColorSpec) -> io::Result<()> {
            self.colors.push(spec.clone());
            Ok(())
        }

        fn reset(&mut self) -> io::Result<()> {
            self.resets += 1;
            Ok(())
        }
    }

    struct MutexedBufferedStream {
        mux: Arc<Mutex<BufferedStream>>,
    }

    impl MutexedBufferedStream {
        fn new() -> MutexedBufferedStream {
            MutexedBufferedStream {
                mux: Arc::new(Mutex::new(BufferedStream::new())),
            }
        }

        fn restore(&mut self) {
            let mux = Arc::clone(&self.mux);
            let mut stream = mux.lock().unwrap();
            stream.restore();
        }

        fn get_lines(&self) -> Vec<Vec<u8>> {
            let mux = Arc::clone(&self.mux);
            let stream = mux.lock().unwrap();
            stream.lines.clone()
        }

        fn get_flushes(&self) -> u8 {
            let mux = Arc::clone(&self.mux);
            let stream = mux.lock().unwrap();
            stream.flushes
        }

        fn get_colors(&self) -> Vec<ColorSpec> {
            let mux = Arc::clone(&self.mux);
            let stream = mux.lock().unwrap();
            stream.colors.clone()
        }

        fn get_resets(&self) -> u8 {
            let mux = Arc::clone(&self.mux);
            let stream = mux.lock().unwrap();
            stream.resets
        }
    }

    impl Clone for MutexedBufferedStream {
        fn clone(&self) -> Self {
            Self {
                mux: self.mux.clone(),
            }
        }
    }

    impl io::Write for MutexedBufferedStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let mux = Arc::clone(&self.mux);
            let mut stream = mux
                .lock()
                .map_err(|e| io::Error::other(format!("could not lock mutex: {:?}", e)))?;
            stream.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            let mux = Arc::clone(&self.mux);
            let mut stream = mux
                .lock()
                .map_err(|e| io::Error::other(format!("could not lock mutex: {:?}", e)))?;
            stream.flush()
        }
    }

    impl WriteColor for MutexedBufferedStream {
        fn supports_color(&self) -> bool {
            true
        }

        fn set_color(&mut self, spec: &ColorSpec) -> io::Result<()> {
            let mux = Arc::clone(&self.mux);
            let mut stream = mux
                .lock()
                .map_err(|e| io::Error::other(format!("could not lock mutex: {:?}", e)))?;
            stream.set_color(spec)
        }

        fn reset(&mut self) -> io::Result<()> {
            let mux = Arc::clone(&self.mux);
            let mut stream = mux
                .lock()
                .map_err(|e| io::Error::other(format!("could not lock mutex: {:?}", e)))?;
            stream.reset()
        }
    }

    #[test]
    fn does_init() {
        let cli = Cli {
            verbosity: Verbosity::default(),
        };
        AppLogger::<StandardStream>::init(&cli);
        assert_eq!(log_enabled!(Level::Trace), false);
        assert_eq!(log_enabled!(Level::Debug), false);
        assert_eq!(log_enabled!(Level::Info), true);
        assert_eq!(log_enabled!(Level::Warn), true);
        assert_eq!(log_enabled!(Level::Error), true);
    }

    #[test]
    fn formats_record() {
        let mut w = BufferedStream::new();

        let record = Record::builder()
            .level(Level::Error)
            .args(format_args!("this is an error"))
            .build();
        w.restore();
        format_record(&mut w, &record).expect("unexpected error while formatting");

        assert_eq!(w.lines, vec![b"this is an error".to_vec(), b"\n".to_vec()]);
        assert_eq!(w.flushes, 1);
        assert_eq!(w.colors, vec![get_error_color()]);

        let record = Record::builder()
            .level(Level::Warn)
            .args(format_args!("this is a warning"))
            .build();
        w.restore();
        format_record(&mut w, &record).expect("unexpected error while formatting");

        assert_eq!(w.lines, vec![b"this is a warning".to_vec(), b"\n".to_vec()]);
        assert_eq!(w.flushes, 1);
        assert_eq!(w.colors, vec![get_warn_color()]);

        let record = Record::builder()
            .level(Level::Info)
            .args(format_args!("this is an info"))
            .build();
        w.restore();
        format_record(&mut w, &record).expect("unexpected error while formatting");

        assert_eq!(w.lines, vec![b"this is an info".to_vec(), b"\n".to_vec()]);
        assert_eq!(w.flushes, 1);
        assert_eq!(w.colors, vec![get_info_color()]);

        let record = Record::builder()
            .level(Level::Debug)
            .args(format_args!("this is a debug"))
            .build();
        w.restore();
        format_record(&mut w, &record).expect("unexpected error while formatting");

        assert_eq!(w.lines, vec![b"this is a debug".to_vec(), b"\n".to_vec()]);
        assert_eq!(w.flushes, 1);
        assert_eq!(w.colors, vec![get_debug_color()]);

        let record = Record::builder()
            .level(Level::Trace)
            .args(format_args!("this is a trace"))
            .build();
        w.restore();
        format_record(&mut w, &record).expect("unexpected error while formatting");

        assert_eq!(w.lines, vec![b"this is a trace".to_vec(), b"\n".to_vec()]);
        assert_eq!(w.flushes, 1);
        assert_eq!(w.colors, vec![get_trace_color()]);
    }

    static SHARED_BUFFER: LazyLock<MutexedBufferedStream> =
        LazyLock::new(|| MutexedBufferedStream::new());

    fn get_buffered_writer() -> MutexedBufferedStream {
        SHARED_BUFFER.clone()
    }
    #[test]
    fn does_logging() {
        let debug_rec = Record::builder()
            .level(Level::Debug)
            .args(format_args!("this is a debug"))
            .build();
        let info_rec = Record::builder()
            .level(Level::Info)
            .args(format_args!("this is an info"))
            .build();
        let error_rec = Record::builder()
            .level(Level::Error)
            .args(format_args!("this is an error"))
            .build();

        let mut logger;
        logger = AppLogger::new(LevelFilter::Debug, get_buffered_writer);

        SHARED_BUFFER.clone().restore();
        assert_eq!(logger.enabled(error_rec.metadata()), true);
        logger.log(&debug_rec);
        assert_eq!(
            SHARED_BUFFER.get_lines(),
            vec![b"this is a debug".to_vec(), b"\n".to_vec()]
        );
        assert_eq!(SHARED_BUFFER.get_flushes(), 1);
        assert_eq!(SHARED_BUFFER.get_colors(), vec![get_debug_color()]);
        assert_eq!(SHARED_BUFFER.get_resets(), 1);
        logger.flush();
        assert_eq!(SHARED_BUFFER.get_flushes(), 1);

        SHARED_BUFFER.clone().restore();
        assert_eq!(logger.enabled(info_rec.metadata()), true);
        logger.log(&info_rec);
        assert_eq!(
            SHARED_BUFFER.get_lines(),
            vec![b"this is an info".to_vec(), b"\n".to_vec()]
        );
        assert_eq!(SHARED_BUFFER.get_flushes(), 1);
        assert_eq!(SHARED_BUFFER.get_colors(), vec![get_info_color()]);
        assert_eq!(SHARED_BUFFER.get_resets(), 1);
        logger.flush();
        assert_eq!(SHARED_BUFFER.get_flushes(), 1);

        SHARED_BUFFER.clone().restore();
        assert_eq!(logger.enabled(error_rec.metadata()), true);
        logger.log(&error_rec);
        assert_eq!(
            SHARED_BUFFER.get_lines(),
            vec![b"this is an error".to_vec(), b"\n".to_vec()]
        );
        assert_eq!(SHARED_BUFFER.get_flushes(), 1);
        assert_eq!(SHARED_BUFFER.get_colors(), vec![get_error_color()]);
        assert_eq!(SHARED_BUFFER.get_resets(), 1);
        logger.flush();
        assert_eq!(SHARED_BUFFER.get_flushes(), 1);

        logger = AppLogger::new(LevelFilter::Info, get_buffered_writer);

        SHARED_BUFFER.clone().restore();
        assert_eq!(logger.enabled(debug_rec.metadata()), false);
        logger.log(&debug_rec);
        assert_eq!(SHARED_BUFFER.get_lines(), Vec::<Vec<u8>>::new());
        assert_eq!(SHARED_BUFFER.get_flushes(), 0);
        assert_eq!(SHARED_BUFFER.get_colors(), Vec::<ColorSpec>::new());
        assert_eq!(SHARED_BUFFER.get_resets(), 0);
        logger.flush();
        assert_eq!(SHARED_BUFFER.get_flushes(), 0);

        SHARED_BUFFER.clone().restore();
        assert_eq!(logger.enabled(info_rec.metadata()), true);
        logger.log(&info_rec);
        assert_eq!(
            SHARED_BUFFER.get_lines(),
            vec![b"this is an info".to_vec(), b"\n".to_vec()]
        );
        assert_eq!(SHARED_BUFFER.get_flushes(), 1);
        assert_eq!(SHARED_BUFFER.get_colors(), vec![get_info_color()]);
        assert_eq!(SHARED_BUFFER.get_resets(), 1);
        logger.flush();
        assert_eq!(SHARED_BUFFER.get_flushes(), 1);

        SHARED_BUFFER.clone().restore();
        assert_eq!(logger.enabled(error_rec.metadata()), true);
        logger.log(&error_rec);
        assert_eq!(
            SHARED_BUFFER.get_lines(),
            vec![b"this is an error".to_vec(), b"\n".to_vec()]
        );
        assert_eq!(SHARED_BUFFER.get_flushes(), 1);
        assert_eq!(SHARED_BUFFER.get_colors(), vec![get_error_color()]);
        assert_eq!(SHARED_BUFFER.get_resets(), 1);
        logger.flush();
        assert_eq!(SHARED_BUFFER.get_flushes(), 1);
    }
}
