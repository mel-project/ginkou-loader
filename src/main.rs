#![windows_subsystem = "windows"]

use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::Context;
use clap::{ArgGroup, Parser};
use tap::Tap;
use tide::listener::Listener;
use wry::{
    application::{
        dpi::LogicalSize,
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    webview::{WebContext, WebViewBuilder},
};

use crate::ipc::IPCRequest;

mod ipc;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[derive(Parser, Clone)]
#[clap(group(
    ArgGroup::new("html_locator")
        .required(true)
        .args(&["dev-port", "html-path"]),
))]
/// Load wallet html and run melwalletd.
pub struct Args {
    ///listen for ginkou on http://localhost:<port>
    #[clap(long)]
    dev_port: Option<u32>,
    /// path to compiled ginkou html
    #[clap(long)]
    html_path: Option<PathBuf>,
    /// path to melwalletd
    #[clap(long, default_value = r#"melwalletd"#)]
    melwalletd_path: PathBuf,
    /// path to persistent data like cookies and Storage
    #[clap(long)]
    data_path: Option<PathBuf>,
    /// path to the wallet
    #[clap(long)]
    wallet_path: Option<PathBuf>,
    #[clap(long)]
    debug_window_open: bool,
}

fn main() -> anyhow::Result<()> {
    let args: Args = {
        let mut args = Args::parse();

        args.wallet_path = Some(args.wallet_path.unwrap_or_else(|| {
            dirs::data_local_dir()
                .expect("no wallet directory")
                .tap_mut(|d| d.push("themelio-wallets"))
        }));

        args.data_path = Some(args.data_path.unwrap_or_else(|| {
            dirs::data_local_dir()
                .expect("no wallet directory")
                .tap_mut(|d| d.push("themelio-wallet-gui-data"))
        }));

        args
    };

    let mwd_auth_token: String = {
        let mut fpath = args.data_path.clone().unwrap();
        fpath.push("auth.txt");
        // use a file, so that even when melwalletd runs off into the background for some reason, everything still works
        if let Ok(existing) = std::fs::read(&fpath) {
            String::from_utf8_lossy(&existing).to_string()
        } else {
            let mut buf = [0u8; 32];
            getrandom::getrandom(&mut buf)?;
            let r = hex::encode(&buf);
            std::fs::write(fpath, r.as_bytes())?;
            r
        }
    };
    eprintln!("auth token = {}", mwd_auth_token);
    std::env::set_var("MELWALLETD_AUTH_TOKEN", &mwd_auth_token);

    let wallet_path = args.wallet_path.clone().unwrap();
    let data_path = args.data_path.clone().unwrap();
    // first, we start a tide-based server that runs off serving the directory
    let html_path = args.html_path.clone();
    let (send_addr, recv_addr) = smol::channel::unbounded();
    let port = args.dev_port;

    let html_addr = match port {
        None => {
            smol::spawn(async move {
                let mut app = tide::new();
                app.at("/").serve_dir(html_path.unwrap()).unwrap();
                let mut listener = app.bind("127.1.2.3:9117").await.unwrap();
                send_addr
                    .send(listener.info()[0].connection().to_string())
                    .await
                    .unwrap();
                listener.accept().await.unwrap()
            })
            .detach();
            smol::future::block_on(recv_addr.recv())?
        }
        Some(_) => format!("http://localhost:{}", port.unwrap()),
    };

    eprintln!("{html_addr}");
    eprintln!("{:?}", args.melwalletd_path.as_os_str());
    // first start melwalletd
    // TODO: start melwalletd with proper options, especially authentication!
    let mut cmd = Command::new(args.melwalletd_path.as_os_str())
        .arg("--wallet-dir")
        .arg(wallet_path.as_os_str())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .tap_mut(|_c| {
            #[cfg(windows)]
            _c.creation_flags(0x08000000);
        })
        .spawn()
        .context("cannot spawn melwalletd")?;
    scopeguard::defer!(cmd.kill().unwrap());
    let script = format!(
        "{}\nwindow.MELWALLETD_AUTH_TOKEN={:?}",
        include_str!("./js/index.js"),
        mwd_auth_token
    );
    let event_loop: EventLoop<()> = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Mellis")
        .with_inner_size(LogicalSize::new(390, 600))
        .with_resizable(true)
        .build(&event_loop)?;

    let webview = WebViewBuilder::new(window)?
        .with_url(&format!("{}/index.html", html_addr))?
        .with_web_context(&mut WebContext::new(Some(data_path)))
        .with_devtools(true)
        .with_ipc_handler(IPCRequest::handler_with_context(args.clone()))
        .with_initialization_script(&script)
        .build()?;

    if args.debug_window_open {
        webview.open_devtools()
    };

    event_loop.run(move |event, _event_loop_window_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(_window) => {
                // webview.resize();
            }
            _ => (),
        }
    });
}
