use serde::{Deserialize, Serialize};
use crate::repair_plan::RepairPlan;
use std::env;


#[derive(Debug, Serialize)]
struct AIRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: AIMessage,
}

#[derive(Debug, Deserialize)]
struct AIMessage {
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAdvisory {
    pub recommended: Option<String>,
    pub priority_order: Vec<String>,
    pub reasoning: Vec<String>,
    pub cautions: Vec<String>,
}

pub fn generate_ai_advisory(plans: Vec<RepairPlan>) -> Option<AIAdvisory> {
    let api_key = env::var("VOXLINUX_AI_KEY").ok()?;

    let plan_summary: Vec<_> = plans.iter().map(|p| {
        serde_json::json!({
            "id": p.id,
            "issue": p.issue,
            "risk": format!("{:?}", p.risk),
                          "reversible": p.reversible,
                          "requires_reboot": p.requires_reboot,
                          "confidence_high": p.confidence_high,
                          "actions": p.actions,
                          "explain": p.explain
        })
    }).collect();

    let prompt = format!(
        "You are an advisory AI for a self-healing Linux system.
        Analyze the following repair plans and return STRICT JSON only in this format:

        {{
        \"recommended\": \"plan-id\",
        \"priority_order\": [\"plan1\", \"plan2\"],
        \"reasoning\": [\"text...\"],
        \"cautions\": [\"text...\"]
}}

Plans:
{}
",
serde_json::to_string_pretty(&plan_summary).unwrap()
    );

    let request = AIRequest {
        model: "llama-3.3-70b-versatile".into(),
        messages: vec![
            Message {
                role: "system".into(),
                content: "You must return valid JSON only. No explanation outside JSON.".into(),
            },
            Message {
                role: "user".into(),
                content: prompt,
            },
        ],
    };

    let client = reqwest::blocking::Client::new();

    let response = client
    .post("https://api.groq.com/openai/v1/chat/completions")
    .bearer_auth(api_key)
    .json(&request)
    .send();

    match response {
        Ok(mut resp) => {
            let text = resp.text().unwrap_or_default();


            return serde_json::from_str::<AIResponse>(&text)
            .ok()
            .and_then(|parsed| {
                parsed.choices.first().and_then(|c| {
                    serde_json::from_str::<AIAdvisory>(&c.message.content).ok()
                })
            });
        }
        Err(e) => {
            println!("AI REQUEST FAILED: {}", e);
            return None;
        }
    }

    println!("KEY LENGTH: {}", api_key.len());
}
