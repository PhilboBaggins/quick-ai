use std::io::stdin;
use clap::{arg, command};
use itertools::Itertools;
use keyring::{Entry, Result};
use openai::completions::Completion;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const KEYRING_ENTRY: &str = "OPENAI_KEY";
const MODEL_NAME: &str = "text-davinci-003";

fn handle_cli() -> Result<(String, String)> {
    let matches = command!()
        .arg(arg!([prompt] ... "Prompt to sent to the AI"))
        .arg(
            arg!(
                -s --store_key <KEY> "Store OpenAI API key in your platform's secure key store"
            )
            .conflicts_with_all(&["delete_key", "print_key"]),
        )
        .arg(
            arg!(
                -d --delete_key "Delete previously saved OpenAI API key"
            )
            .conflicts_with_all(&["store_key", "print_key"]),
        )
        .arg(
            arg!(
                -p --print_key "Print previously saved OpenAI API key"
            )
            .conflicts_with_all(&["store_key", "delete_key"]),
        )
        .get_matches();

    let entry = Entry::new(APP_NAME, KEYRING_ENTRY)?;

    if let Some(new_api_key) = matches.get_one::<String>("store_key") {
        entry.set_password(new_api_key)?;
        println!("OpenAI API key stored");
        std::process::exit(0);
    }

    if matches.get_flag("delete_key") {
        entry.delete_password()?;
        println!("OpenAI API key deleted");
        std::process::exit(0);
    }

    let openai_api_key = match entry.get_password() {
        Ok(key) => key,
        Err(_) => {
            eprintln!("OpenAI API key not found. Please enter it using the --store_key flag.");
            std::process::exit(1);
        }
    };

    if matches.get_flag("print_key") {
        println!("OpenAI API key: {}", openai_api_key);
        std::process::exit(0);
    }

    // Get prompt from CLI or stdin
    let prompt = match matches.get_many::<String>("prompt") {
        Some(prompts) => prompts.map(|s| s.as_str()).join(" "),
        None => {
            println!("Please enter a prompt to send to the AI.");
            let mut prompt = String::new();
            stdin().read_line(&mut prompt).unwrap();
            println!("");
            prompt
        }
    };        

    Ok((openai_api_key, prompt))
}

async fn ask_ai_a_question(prompt: &str, model_name: &str) -> Result<String> {
    let prompt = format!("{} -- short concise answer", prompt);

    let completion = Completion::builder(model_name)
        .prompt(&prompt)
        .max_tokens(1024)
        .create()
        .await
        .unwrap();

    let answer = completion
        .choices
        .iter()
        .map(|choice| choice.text.to_string())
        .join("\n\n");

    Ok(answer)
}

#[tokio::main]
async fn main() -> Result<()> {
    let (openai_api_key, prompt) = handle_cli()?;

    println!("Question: {}", prompt);

    openai::set_key(openai_api_key);
    let answer = ask_ai_a_question(&prompt, MODEL_NAME).await?;

    println!("Answer: {}", answer);

    Ok(())
}
