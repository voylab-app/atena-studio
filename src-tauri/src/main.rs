// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "atena", about = "Atena Studio - AI Cognitive Platform")]
struct Cli {
    /// Run as headless web server (accessible from web browsers)
    #[arg(long, env = "ATENA_SERVER")]
    server: bool,

    /// Host address to bind the web server (default: 127.0.0.1)
    #[arg(long, env = "ATENA_HOST", default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on in server mode (default: 7860)
    #[arg(long, env = "ATENA_PORT", default_value_t = 7860)]
    port: u16,

    /// Authentication token for web server access (optional, auto-generated if omitted)
    #[arg(long, env = "ATENA_TOKEN")]
    token: Option<String>,

    /// Path to frontend dist directory
    #[arg(long, env = "ATENA_STATIC_DIR")]
    static_dir: Option<String>,
}

fn main() {
    // When launched as a desktop app on macOS, Finder can pass -psn_... args.
    // Try to parse CLI args, or fallback to default desktop launch if unrecognized args.
    let cli = match Cli::try_parse() {
        Ok(args) => args,
        Err(_) => {
            app_lib::run();
            return;
        }
    };

    if cli.server {
        let server_args = app_lib::server::ServerArgs {
            host: cli.host,
            port: cli.port,
            token: cli.token,
            static_dir: cli.static_dir,
        };

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to initialize Tokio runtime for Atena server");

        if let Err(e) = rt.block_on(app_lib::server::run_server(server_args)) {
            eprintln!("Atena Studio server error: {}", e);
        }
    } else {
        app_lib::run();
    }
}
