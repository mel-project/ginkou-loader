

use serde::{Deserialize};
use wry::application::{window::Window, dpi::PhysicalSize};

#[derive(Debug, Deserialize)]
#[serde(tag = "_event", rename_all="snake_case")]
pub enum IPCRequest{
    SetConversionFactor { conversion_factor: f64 },
    DownloadLogs,
    #[serde(skip)] 
    Unknown(String)
}

impl IPCRequest {
    pub fn handler(window: &Window, request: String){
            // let request = request.clone().replace("\"", "");
            let ipc: IPCRequest = serde_json::from_str(&request).unwrap_or(IPCRequest::Unknown(request));
            eprintln!("Request: {ipc:?}");
            match ipc {
                IPCRequest::SetConversionFactor { conversion_factor }  => {
                    
                    let factor: f64 = conversion_factor;
                    
                    // eprintln!("SET CONVERSION FACTOR {}", factor);
                    
                    window.set_inner_size(PhysicalSize {
                        width: 400.0 * factor,
                        height: 700.0 * factor,
                    });
                    window.set_resizable(false);
                    // Some(RpcResponse::new_result(req.id, Some(serde_json::to_value(0.0f64).unwrap())))
                }
                IPCRequest::DownloadLogs => {
                    // let mut data;
                    // let future = async {
                    //     let file = AsyncFileDialog::new()
                    //         .add_filter("text", &["txt", "rs"])
                    //         .add_filter("rust", &["rs", "toml"])
                    //         .set_directory("/")
                    //         .pick_file()
                    //         .await;
                    
                    //     data = file.unwrap().read().await;
                    // };
                    // println!("{data}");
                    // None
                }
                IPCRequest::Unknown(_) => ()
            
        }
    }
}