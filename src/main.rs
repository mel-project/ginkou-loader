use std::{path::PathBuf, process::Command};

use anyhow::Context;
use argh::FromArgs;
use tap::Tap;
use tide::listener::Listener;
use wry::{
    application::{
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

    let wallet_path = args.wallet_path.clone().unwrap_or_else(|| {
        dirs::data_local_dir()
            .expect("no wallet directory")
            .tap_mut(|d| d.push("themelio-wallets"))
    });
    let data_path = args.data_path.clone().unwrap_or_else(|| {
        dirs::data_local_dir()
            .expect("no wallet directory")
            .tap_mut(|d| d.push("themelio-wallet-gui-data"))
    });

    // first, we start a tide-based server that runs off serving the directory
    let html_path = args.html_path.clone();
    let (send_addr, recv_addr) = smol::channel::unbounded();
    smol::spawn(async move {
        let mut app = tide::new();
        app.at("/").serve_dir(html_path).unwrap();
        let mut listener = app.bind("127.0.0.123:12345").await.unwrap();
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

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Ginkou")
        .build(&event_loop)?;
    let webview = WebViewBuilder::new(window)?
        // .with_custom_protocol("wry".to_string(), move |_, url| {
        //     let url = url.replace("wry://localhost/", "");
        //     let mut path = args.html_path.clone();
        //     path.push(url);
        //     dbg!(&path);
        //     Ok(std::fs::read(&path)?)
        // })
        .with_url(&format!("{}/index.html", html_addr))?
        .with_web_context(&mut WebContext::new(Some(data_path)))
        .build()?;

    event_loop.run(move |event, _, control_flow| {
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
            Event::RedrawRequested(window) => {
                webview.resize().expect("cannot resize webview");
            }
            _ => (),
        }
    });
}
