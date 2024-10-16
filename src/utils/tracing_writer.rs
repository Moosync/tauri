use core::str;
use std::io::{self, Write};

use serde::Serialize;
use tracing_subscriber::fmt::MakeWriter;
use wasm_bindgen_futures::spawn_local;

use crate::{console_debug, console_error, console_info, console_trace, console_warn};

use super::common::invoke;

#[derive(Clone, Copy, Debug, Default)]
pub struct MakeConsoleWriter {
    log_file: bool,
}

impl MakeConsoleWriter {
    pub fn new_log_file() -> Self {
        Self { log_file: true }
    }
}

impl<'a> MakeWriter<'a> for MakeConsoleWriter {
    type Writer = ConsoleWriter;

    #[tracing::instrument(level = "trace", skip(self))]
    fn make_writer(&'a self) -> Self::Writer {
        ConsoleWriter {
            level: tracing::Level::DEBUG,
            data: vec![],
            log_file: self.log_file,
        }
    }

    #[tracing::instrument(level = "trace", skip(self, meta))]
    fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        ConsoleWriter {
            level: *meta.level(),
            data: vec![],
            log_file: self.log_file,
        }
    }
}

pub struct ConsoleWriter {
    level: tracing::Level,
    data: Vec<u8>,
    log_file: bool,
}

#[derive(Serialize)]
struct RendererWriteArgs {
    data: Vec<u8>,
}

impl Write for ConsoleWriter {
    #[tracing::instrument(level = "trace", skip(self, buf))]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.write(buf)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn flush(&mut self) -> io::Result<()> {
        if !self.log_file {
            let parsed = str::from_utf8(&self.data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            match self.level {
                tracing::Level::DEBUG => {
                    console_debug!("{}", parsed);
                }
                tracing::Level::ERROR => {
                    console_error!("{}", parsed);
                }
                tracing::Level::INFO => {
                    console_info!("{}", parsed);
                }
                tracing::Level::TRACE => {
                    console_trace!("{}", parsed);
                }
                tracing::Level::WARN => {
                    console_warn!("{}", parsed);
                }
            };
        } else {
            let data = self.data.clone();
            spawn_local(async move {
                if let Err(e) = invoke(
                    "renderer_write",
                    serde_wasm_bindgen::to_value(&RendererWriteArgs { data }).unwrap(),
                )
                .await
                {
                    console_error!("Failed to write log: {:?}", e);
                }
            });
        }

        Ok(())
    }
}

impl Drop for ConsoleWriter {
    #[tracing::instrument(level = "trace", skip(self))]
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
