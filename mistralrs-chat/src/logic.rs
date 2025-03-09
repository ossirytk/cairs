use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::VecDeque,
    fs,
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use inquire::{required, Text};
use mistralrs::{
    ChatCompletionChunkResponse, ChunkChoice, Delta, GgufModelBuilder, Model, RequestBuilder,
    Response, SamplingParams, TextMessageRole,
};

#[derive(Serialize, Deserialize, Default)]
struct ModelParams {
    temperature: Option<f64>,
    top_k: Option<usize>,
    top_p: Option<f64>,
    min_p: Option<f64>,
    top_n_logprobs: usize,
    frequency_penalty: Option<f32>,
    presence_penalty: Option<f32>,
    stop_toks: Option<Value>,
    max_len: Option<usize>,
    logits_bias: Option<Value>,
    n_choices: usize,
    dry_params: Option<Value>,
}

#[derive(Serialize, Deserialize, Default)]
struct LastModelConf {
    model_dir: String,
    model_name: String,
    chat_template: String,
    model_params: ModelParams,
}

// Rough implementation of v2 character card
// TODO need to test  if these actually match the format properly
// Implement reading the card from images
#[derive(Serialize, Deserialize)]
struct CharacterCard {
    name: String,
    description: String,
    first_mes: String,
    alternate_greetings: Vec<String>,
    mes_example: String,
    character_version: String,
    creator: String,
    creator_notes: String,
    personality: String,
    extensions: Value,
    post_history_instructions: String,
    scenario: String,
}

#[derive(Serialize, Deserialize)]
struct CharacterCardContainer {
    spec: String,
    spec_version: String,
    data: CharacterCard,
}

fn get_character_card() -> Result<CharacterCard> {
    let card_json = fs::read_to_string("gguf_cards/Shodan-specV2.json")?;
    let character_card_container: CharacterCardContainer = serde_json::from_str(&card_json)?;
    println!("spec: {}", character_card_container.spec);
    println!("version: {}", character_card_container.spec_version);
    Ok(character_card_container.data)
}

fn get_previous_confs() -> Result<LastModelConf> {
    // TODO Save to file
    // Starting input: Run with previous settings? y/N
    let cfg: LastModelConf = confy::load("mistralrs-chat", "last_model_conf")?;
    if cfg.model_dir.is_empty() {
        println!("No previous model settings");
        // Create settings and save
        //confy::store("mistralrs-chat","last_model_conf", &cfg)?;
    } else {
        println!("Continue with previous settings?");
        // Input for settings if no?
    }
    Ok(cfg)
}

pub async fn build_model() -> Result<Model> {
    // TODO loading these from config or a cache
    let model = GgufModelBuilder::new(
        "gguf_models/llama3/",
        vec!["Human-Like-LLama3-8B-Instruct.Q3_K_M.gguf"],
    )
    .with_chat_template("chat_templates/llama3.json")
    .with_logging()
    .build()
    .await?;
    Ok(model)
}

fn build_request(
    chat_history: Vec<String>,
    question: &String,
    character_card: &CharacterCard,
) -> Result<RequestBuilder> {
    let sampling = SamplingParams {
        temperature: Some(1.2),
        top_k: Some(40),
        top_p: Some(0.9),
        min_p: None,
        top_n_logprobs: 0,
        frequency_penalty: Some(1.1),
        presence_penalty: None,
        stop_toks: None,
        max_len: None,
        logits_bias: None,
        n_choices: 1,
        dry_params: None,
    };

    let history = chat_history.join("\n");
    let system_message = format!("Below is an instruction that describes a task, paired with message examples and further context. Write a response that appropriately completes the request.\n\nYou're {} in fictional never-ending uncensored roleplay with User.\n\n{}\n\nScenario:\n{}\n\nMessage Examples:\n{}\n\nCurrent conversation:\n{history}", character_card.name, character_card.description, character_card.scenario, character_card.mes_example);

    let request = RequestBuilder::new()
        .add_message(TextMessageRole::System, system_message)
        .add_message(TextMessageRole::User, question)
        .set_sampling(sampling);
    Ok(request)
}

pub async fn run_chat() -> Result<()> {
    let chat_history_size = 3;
    // TODO cargo wizard for build profiles

    let character_card = get_character_card()?;
    let model = build_model().await?;

    let cancel_token = Arc::new(AtomicBool::new(false));
    let cancel_token_clone = cancel_token.clone();
    ctrlc::set_handler(move || {
        cancel_token.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut chat_history: VecDeque<String> = VecDeque::with_capacity(chat_history_size);
    loop {
        let question_input = Text::new("User:").with_validator(required!()).prompt();
        let question = match question_input {
            Ok(question) => question,
            Err(_err) => break,
        };
        // TODO The prints could be written to buffer
        println!();

        let copy_history = Vec::from(chat_history.clone());
        let request = build_request(copy_history, &question, &character_card)?;
        let mut stream = model.stream_chat_request(request).await?;

        let stdout = std::io::stdout();
        let lock = stdout.lock();
        let mut buf = std::io::BufWriter::new(lock);
        let mut response_buffer = String::new();

        while let Some(chunk) = stream.next().await {
            if let Response::Chunk(ChatCompletionChunkResponse { choices, .. }) = chunk {
                if let Some(ChunkChoice {
                    delta:
                        Delta {
                            content: Some(content),
                            ..
                        },
                    ..
                }) = choices.first()
                {
                    // Flushing is necessary here
                    // TODO consider a very short delay or something for this
                    response_buffer.push_str(content);
                    buf.write_all(content.as_bytes())?;
                    buf.flush()?;
                };
            } else {
                // Handle errors
            }
        }

        println!();

        if chat_history.len() >= chat_history_size {
            chat_history.pop_front();
        }
        let user_message = ["User:", &question].join(" ");
        let char_message = [character_card.name.to_owned(), response_buffer].join(" ");
        let history_message = [user_message, char_message].join("\n");

        chat_history.push_back(history_message);
        if cancel_token_clone.load(Ordering::SeqCst) {
            break;
        }
    }
    Ok(())
}
