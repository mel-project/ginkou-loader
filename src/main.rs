use std::{path::PathBuf, process::Command};

use anyhow::Context;
use argh::FromArgs;
use wry::{
    application::{
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    webview::WebViewBuilder,
};

#[derive(FromArgs)]
/// Load wallet html and run melwalletd.
struct Args {
    /// path to compiled ginkou html
    #[argh(option)]
    html_path: PathBuf,
    /// path to melwalletd
    #[argh(option, default = "\"melwalletd\".into()")]
    melwalletd_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let mut wallets_dir = dirs::home_dir().context("cannot obtain home directory")?;
    wallets_dir.push(".gk-melwallet");

    let args: Args = argh::from_env();

    // first start melwalletd
    // TODO: start melwalletd with proper options, especially authentication!
    let mut cmd = Command::new(args.melwalletd_path.as_os_str())
        .arg("--wallet-dir")
        .arg(wallets_dir.as_os_str())
        .spawn()
        .context("cannot spawn melwalletd")?;
    scopeguard::defer!(cmd.kill().unwrap());

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Hello World")
        .build(&event_loop)?;
    let _webview = WebViewBuilder::new(window)?
        .with_custom_protocol("wry".to_string(), move |_, url| {
            let url = url.replace("wry:///", "");
            let mut path = args.html_path.clone();
            path.push(url);
            dbg!(&path);
            Ok(std::fs::read(&path)?)
        })
        .with_url("wry:///index.html")?
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
