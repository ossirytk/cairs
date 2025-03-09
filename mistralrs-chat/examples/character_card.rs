use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

fn main() -> Result<()> {
    let card_json = fs::read_to_string("gguf_cards/Shodan-specV2.json")?;
    let character_card_container: CharacterCardContainer = serde_json::from_str(&card_json)?;
    println!("spec: {}", character_card_container.spec);
    println!("version: {}", character_card_container.spec_version);
    let character_card: CharacterCard = character_card_container.data;
    println!("{}", character_card.name);
    println!("{}", character_card.description);
    println!("{}", character_card.first_mes);

    Ok(())
}
