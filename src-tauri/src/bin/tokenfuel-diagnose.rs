// Explicit operator consent is required before reading an existing local session.
fn main() {
    if !std::env::args().any(|a| a == "--consent-local-session") {
        eprintln!(
            "Read-only Codex quota diagnostic. Run with --consent-local-session to authorize the installed Codex session. No conversations or model requests are accessed."
        );
        std::process::exit(2);
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    match runtime.block_on(tokenfuel_app::diagnose_codex()) {
        Ok(s) => {
            println!("{s}");
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
