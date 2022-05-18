use std::{env::join_paths, fs::File, io::Write, path::PathBuf, sync::Mutex};

use crate::Args;
use log::error;
use rfd::FileDialog;
use serde::Deserialize;
use std::fs;
use wry::application::{dpi::LogicalSize, window::Window};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]

pub enum LogLevel {
    Log,
    Debug,
    Info,
    Warn,
    Error,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "_event", rename_all = "kebab-case")]
pub enum IPCRequest {
    SetConversionFactor {
        conversion_factor: f64,
    },
    DownloadLogs,
    LogEvent {
        level: LogLevel,
        message: String,
    },
    OpenBrowser {
        url: String,
    },
    #[serde(skip)]
    Unknown(String),
}
pub struct IPCContext {
    ginkou_logs_path: PathBuf,
    melwalletd_logs_path: PathBuf,
    ginkou_logs: Mutex<File>,
    melwalletd_logs: Mutex<File>,
}

impl IPCContext {
    pub fn from_args(args: Args) -> std::io::Result<IPCContext> {
        let ginkou_logs_path = args.data_path.join("ginkou.log");
        let melwalletd_logs_path = args.data_path.join("melwalletd.log");
        let context = IPCContext {
            ginkou_logs: Mutex::new(File::create(ginkou_logs_path.clone())?),
            melwalletd_logs: Mutex::new(File::create(melwalletd_logs_path.clone())?),
            ginkou_logs_path,
            melwalletd_logs_path,
        };
        Ok(context)
    }
}
impl IPCRequest {
    pub fn handler_with_context(context: IPCContext) -> impl Fn(&Window, String) {
        move |window: &Window, request: String| {
            // println!("{request}");
            let ipc: IPCRequest =  serde_json::from_str(&request).unwrap_or(IPCRequest::Unknown(request));
            match ipc {
                IPCRequest::SetConversionFactor { conversion_factor } => {
                    // window.set_resizable(true);
                    let factor = conversion_factor / 0.95;
                    eprintln!("SET CONVERSION FACTOR {}", factor);
                    window.set_inner_size(LogicalSize {
                        width: 390.0 * factor,
                        height: 600.0 * factor,
                    });
                    window.set_resizable(false);
                }
                IPCRequest::DownloadLogs => {
                    let mellis_logs = FileDialog::new()
                        .set_directory("~/")
                        .pick_folder()
                        .unwrap_or_default()
                        .join("mellis_logs");

                    fs::create_dir(&mellis_logs).unwrap();
                    fs::copy(context.melwalletd_logs_path.clone(), mellis_logs.clone().join("melwalletd.log")).unwrap();
                    fs::copy(context.ginkou_logs_path.clone(), mellis_logs.join("ginkou.log")).unwrap();

                    // smol::future::block_on(logs.await);
                }
                IPCRequest::LogEvent { level, message } => {
                    writeln!(
                        context.ginkou_logs.lock().unwrap(),
                        "{:?}: {}",
                        level,
                        message
                    )
                    .unwrap();
                }
                IPCRequest::OpenBrowser { url } => {
                    let _ = webbrowser::open(&url);
                }
                IPCRequest::Unknown(_) => (),

            };
        }
    }
}
