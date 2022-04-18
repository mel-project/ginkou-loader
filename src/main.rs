use std::{path::PathBuf, process::Command};

use anyhow::Context;
use argh::FromArgs;
use smol::channel;
use tap::Tap;
use tide::listener::Listener;
use tide::Server;
use wry::{
    application::{
        dpi::PhysicalSize,
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    webview::{WebContext, WebViewBuilder},
};

#[derive(FromArgs)]
/// Load wallet html and run melwalletd.
struct Args {
    /// path to compiled ginkou html
    #[argh(option)]
    html_path: PathBuf,
    /// path to melwalletd
    #[argh(option, default = r#""melwalletd".into()"#)]
    melwalletd_path: PathBuf,
    /// path to persistent data like cookies and Storage
    #[argh(option)]
    data_path: Option<PathBuf>,
    /// path to the wallet
    #[argh(option)]
    wallet_path: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();

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
    let html_path: PathBuf = args.html_path.clone();

    let (send_addr, recv_addr): (channel::Sender<String>, channel::Receiver<String>) = smol::channel::unbounded();

    smol::spawn(async move {
        let mut app: Server<()> = tide::new();

        app.at("/").serve_dir(html_path).expect("Could not serve from HTML path.");

        let mut listener = app.bind("127.0.0.1:9117").await.expect("Could not bind a server at 127.0.0.1 on port 9117.");

        send_addr
            .send(listener.info()[0].connection().to_string())
            .await
            .unwrap();

        listener.accept().await.unwrap()
    })
    .detach();


    let html_addr = smol::future::block_on(recv_addr.recv())?;


    // first start melwalletd
    // TODO: start melwalletd with proper options, especially authentication!
    let mut cmd = Command::new(args.melwalletd_path.as_os_str())
        .arg("--wallet-dir")
        .arg(wallet_path.as_os_str())
        .spawn()
        .context("cannot spawn melwalletd")?;

    let event_loop: EventLoop<()> = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Mellis")
        .with_inner_size(PhysicalSize::new(400, 700))
        .build(&event_loop)?;

    let webview = WebViewBuilder::new(window)?
        .with_url(&format!("{}/index.html", html_addr))?
        .with_web_context(&mut WebContext::new(Some(data_path)))
        .with_ipc_handler(|window, request| {
            match request.as_str() {
                "set_conversion_factor" => {
                    // let convfact: (f64,) = serde_json::from_value(request.params.unwrap()).unwrap();
                    //
                    // let factor: f64 = convfact.0;
                    //
                    // eprintln!("SET CONVERSION FACTOR {}", factor);
                    //
                    // window.set_inner_size(PhysicalSize {
                    //     width: 400.0 * factor,
                    //     height: 700.0 * factor,
                    // });

                    window.set_resizable(false);
                },
                _ => panic!("Method did not match.")
            }
        })
        .with_initialization_script(r"
        window.onload = function() {
            window.rpc.call('set_conversion_factor', parseFloat(getComputedStyle(document.documentElement).fontSize) / 16);
        }
        ")
        .build()?;

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