use rfd::{FileDialog};
use serde::Deserialize;
use wry::application::{dpi::LogicalSize, window::Window};

use crate::Args;

#[derive(Debug, Deserialize)]
#[serde(tag = "_event", rename_all = "kebab-case")]
pub enum IPCRequest {
    SetConversionFactor {
        conversion_factor: f64,
    },
    DownloadLogs,
    #[serde(skip)]
    Unknown(String),
}

impl IPCRequest {
    pub fn handler_with_context(args: Args) -> impl Fn(&Window, String) -> () {
        move |window: &Window, request: String| {
            // let request = request.clone().replace("\"", "");
            let ipc: IPCRequest =
                serde_json::from_str(&request).unwrap_or(IPCRequest::Unknown(request));
            eprintln!("Request: {ipc:?}");
            match ipc {
                IPCRequest::SetConversionFactor { conversion_factor } => {
                    let factor = conversion_factor/0.95;
                    eprintln!("SET CONVERSION FACTOR {}", factor);
                    window.set_inner_size(LogicalSize {
                        width: 420.0 * factor,
                        height: 800.0 * factor,
                    }); 
                    window.set_resizable(false);
                    
                }
                IPCRequest::DownloadLogs => {
                    let file = FileDialog::new()
                        .set_directory("~/")
                        .pick_folder()
                        .unwrap_or_default();

                    println!("{file:?}");
                    // smol::future::block_on(future.await);
                }
                IPCRequest::Unknown(_) => (),
            };
        }
    }
}
