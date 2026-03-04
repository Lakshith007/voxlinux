use serde::{Deserialize, Serialize};
use voxlinux::repair_plan::RepairPlan;
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
    println!("[AI DEBUG] generate_ai_advisory() CALLED");

    let api_key = env::var("GROQ_API_KEY").ok()?;
    println!("[AI DEBUG] API key detected");

    let plan_summary: Vec<_> = plans.iter().map(|p| {
        serde_json::json!({
            "id": p.id,
            "issue": p.issue,
            "risk": format!("{:?}", p.risk),
                          "reversible": p.reversible,
                          "requires_reboot": p.requires_reboot,
                          "confidence_high": p.confidence_high,
                          "actions": p.actions
        })
    }).collect();

    let prompt = format!(
        "You are an advisory AI for a self-healing Linux system.
        Return ONLY valid raw JSON.
        Do NOT include markdown.
        Do NOT include explanations outside JSON.

        Return exactly this format:

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
        model: "openai/gpt-oss-120b".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "Return JSON only.".to_string(),
            },
            Message {
                role: "user".to_string(),
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

            println!("[AI DEBUG] RAW RESPONSE:\n{}\n", text);

            let parsed: AIResponse = serde_json::from_str(&text).ok()?;

            let content = &parsed.choices.first()?.message.content;

            println!("[AI DEBUG] MESSAGE CONTENT:\n{}\n", content);

            // 🔥 Extract JSON safely even if model adds extra text
            let start = content.find('{')?;
            let end = content.rfind('}')?;
            let json_slice = &content[start..=end];

            match serde_json::from_str::<AIAdvisory>(json_slice) {
                Ok(advisory) => {
                    println!("[AI DEBUG] AI JSON parsed successfully");
                    Some(advisory)
                }
                Err(e) => {
                    println!("[AI DEBUG] Failed to parse AI JSON: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            println!("[AI DEBUG] AI REQUEST FAILED: {}", e);
            None
        }
    }
}
