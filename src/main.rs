#![windows_subsystem = "windows"]

use std::{path::PathBuf, process::{Command, Stdio}};

use anyhow::Context;
use tap::Tap;
use tide::listener::Listener;
use clap::{Parser, ArgGroup};
use wry::{
    application::{
        dpi::{ LogicalSize},
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
    #[clap(long,default_value = r#"melwalletd"#)]
    melwalletd_path: PathBuf,
    /// path to persistent data like cookies and Storage
    #[clap(long)]
    data_path: Option<PathBuf>,
    /// path to the wallet
    #[clap(long)]
    wallet_path: Option<PathBuf>,
    #[clap(long)]
    debug_window_open: bool,
    #[clap(long)]
    devtools: bool,

    
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let wallet_path: PathBuf = args.wallet_path.clone().unwrap_or_else(|| {
        dirs::data_local_dir()
            .expect("no wallet directory")
            .tap_mut(|d| d.push("themelio-wallets"))
    });
    let data_path: PathBuf = args.data_path.clone().unwrap_or_else(|| {
        dirs::data_local_dir()
            .expect("no wallet directory")
            .tap_mut(|d| d.push("themelio-wallet-gui-data"))
    });

    // first, we start a tide-based server that runs off serving the directory
    let html_path = args.html_path.clone();
    let (send_addr, recv_addr) = smol::channel::unbounded();
    let port = args.dev_port;

    let html_addr = match port.clone() {

        None => {
            smol::spawn(async move {
                let mut app = tide::new();
                app.at("/").serve_dir(html_path.unwrap()).unwrap();
                let mut listener = app.bind("127.0.0.1:9117").await.unwrap();
                send_addr
                    .send(listener.info()[0].connection().to_string())
                    .await
                    .unwrap();
                listener.accept().await.unwrap()
            })
            .detach();
            smol::future::block_on(recv_addr.recv())?
        },
        Some(_) =>  format!("http://localhost:{}", port.unwrap())
    };


    eprintln!("{html_addr}");
    eprintln!("{:?}", args.melwalletd_path.clone().as_os_str());
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
    let script = include_str!("./js/index.js");
    let event_loop: EventLoop<()> = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Mellis")
        .with_inner_size(LogicalSize::new(420, 800))
        .build(&event_loop)?;

    let webview = WebViewBuilder::new(window)?
        .with_url(&format!("{}/index.html", html_addr))?
        .with_web_context(&mut WebContext::new(Some(data_path)))
        .with_devtools(args.devtools)
        .with_ipc_handler(IPCRequest::handler_with_context(args.clone()))
        .with_initialization_script(script)
        .build()?;

    if args.debug_window_open { webview.open_devtools() } ;

    event_loop.run(move |event, _event_loop_window_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                scopeguard::defer!(cmd.kill().unwrap());
                *control_flow = ControlFlow::Exit
            }
            Event::RedrawRequested(_window) => {
                webview.resize().expect("cannot resize webview");
            }
            _ => (),
        }
    });
}