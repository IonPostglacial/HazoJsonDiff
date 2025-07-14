use hazojsondiff::diff_json_strs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <old_dataset.json> <new_dataset.json>", args[0]);
        std::process::exit(1);
    }

    let old_json = std::fs::read_to_string(&args[1]).unwrap_or_else(|e| {
        eprintln!("Failed to read old dataset: {}", e);
        std::process::exit(1);
    });
    let new_json = std::fs::read_to_string(&args[2]).unwrap_or_else(|e| {
        eprintln!("Failed to read new dataset: {}", e);
        std::process::exit(1);
    });

    match diff_json_strs(&old_json, &new_json) {
        Ok(diff_str) => println!("{}", diff_str),
        Err(e) => {
            eprintln!("Failed to diff datasets: {}", e);
            std::process::exit(1);
        }
    }
}