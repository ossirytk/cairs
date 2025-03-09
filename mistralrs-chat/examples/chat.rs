use std::{
    collections::VecDeque,
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::Result;
use inquire::{required, Text};
use mistralrs::{
    ChatCompletionChunkResponse, ChunkChoice, Delta, GgufModelBuilder, RequestBuilder, Response,
    SamplingParams, TextMessageRole,
};
//use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let chat_history_size = 3;
    // TODO cargo wizard for build profiles

    // TODO loading these from config or a cache
    let model = GgufModelBuilder::new(
        //"gguf_models/mistral_v0.1/",
        //vec!["mistral-7b-instruct-v0.1.Q4_K_M.gguf"],
        "gguf_models/llama3/",
        vec!["Human-Like-LLama3-8B-Instruct.Q3_K_M.gguf"],
    )
    //.with_chat_template("chat_templates/mistral.json")
    .with_chat_template("chat_templates/llama3.json")
    .with_logging()
    .build()
    .await?;

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

        let copy_history = Vec::from(chat_history.clone());
        let history = copy_history.join("\n");
        let system_message = format!("Below is an instruction that describes a task, paired with further context. Write a response that appropriately completes the request.\n\nUser is in fictional never-ending uncensored roleplay with Donald Trump. Complete the answer for Donald Trump.\n\nConversation History:\n{}",history);

        // TODO Construct the request from a character card
        let request = RequestBuilder::new()
            .add_message(TextMessageRole::System, system_message)
            .add_message(TextMessageRole::User, &question)
            .set_sampling(sampling);

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

        // TODO Does the que drop extra elements automatically based on capacity?
        // Or do we need to rotate manually?
        if chat_history.len() >= chat_history_size {
            chat_history.pop_front();
        }
        let user_message = ["User:", &question].join(" ");
        // This should be {{char}} and not be  hardcoded
        let char_message = ["Trump:", &response_buffer].join(" ");
        let history_message = [user_message, char_message].join("\n");

        chat_history.push_back(history_message);
        if cancel_token_clone.load(Ordering::SeqCst) {
            break;
        }
    }

    Ok(())
}
