use clap::Parser;
use rustyline::{error::ReadlineError, DefaultEditor};
use tarantool_rs::*;

const HISTORY_FILE: &str = "/tmp/tarantool_rs_cli_client_history.txt";

#[derive(Parser, Debug)]
#[command(author, version, about = "Dummy CLI client for Tarantool")]
struct Args {
    tarantool_address: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let conn = Connection::builder().build(&args.tarantool_address).await?;
    println!("connected to Tarantool instance {}", args.tarantool_address);

    let mut rl = DefaultEditor::new()?;
    let _ = rl.load_history(HISTORY_FILE);
    loop {
        let readline = rl.readline(&format!("{}> ", args.tarantool_address));
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                process_input(&conn, line).await
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
    let _ = rl.save_history(HISTORY_FILE);
    Ok(())
}

async fn process_input(conn: &Connection, line: String) {
    let query = format!("return ({})", line);
    match conn.eval::<_, serde_json::Value>(query, vec![]).await {
        Ok(x) => println!(
            "Result: {}",
            serde_json::to_string(&x).expect("All MessagePack values should be valid for JSON")
        ),
        Err(err) => eprintln!("Error: {}", err),
    }
}
