mod config;
use config::Config;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 || args[1] != "check" {
        eprintln!("usage: llm-gateway check <config.toml> <api-key>");
        std::process::exit(2);
    }

    let cfg = Config::load(&args[2]);
    let Some(key) = cfg.find_key(&args[3]) else {
        println!("unknown key");
        return;
    };
    let Some(project) = cfg.project_for(key) else {
        println!("key #{} points at a missing project", key.id);
        return;
    };

    println!(
        "key #{} -> project {:?}, providers: {:?}",
        key.id, project.name, key.providers
    );
}
