use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

fn main() -> Result<()> {
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
    Ok(())
}
