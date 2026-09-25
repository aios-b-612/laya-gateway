use std::collections::BTreeMap;

use regex::Regex;
use serde_json::{json, Value};

use crate::laya::{Answer, Question, SystemOneRequest};

pub const NO_TOOL: &str = "__no_tool__";
pub const MAX_TOOLS: usize = 32;

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum Decision {
    /// Force the LLM to call this function (fill args only).
    Forced { tool: String, confidence: f64 },
    /// Confident that no tool is needed.
    None { confidence: f64 },
    /// Leave the request untouched.
    Passthrough { reason: String },
}

#[derive(Debug, Clone)]
pub struct DecideInput {
    pub system: String,
    pub last_user: String,
    pub tools: Vec<ToolInfo>,
    pub tool_choice_decided: bool,
}

pub fn skip_reason(input: &DecideInput) -> Option<&'static str> {
    if input.tool_choice_decided {
        return Some("tool_choice_already_decided");
    }
    if input.tools.is_empty() {
        return Some("no_tools");
    }
    if input.tools.len() > MAX_TOOLS {
        return Some("too_many_tools");
    }
    if input.last_user.trim().is_empty() && input.system.trim().is_empty() {
        return Some("no_messages");
    }
    let re = Regex::new(r"^[\p{L}\p{N}_.:/-]{1,128}$").expect("regex");
    for tool in &input.tools {
        if !re.is_match(&tool.name) {
            return Some("unsafe_tool_name");
        }
        if tool.name == NO_TOOL {
            return Some("reserved_tool_name");
        }
    }
    let mut names: Vec<_> = input.tools.iter().map(|t| t.name.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    if names.len() != input.tools.len() {
        return Some("duplicate_tool_names");
    }
    None
}

pub fn build_request(input: &DecideInput, model: Option<String>) -> SystemOneRequest {
    let mut criteria = BTreeMap::new();
    for tool in &input.tools {
        let desc = if tool.description.is_empty() {
            None
        } else {
            Some(tool.description.clone())
        };
        criteria.insert(tool.name.clone(), desc);
    }
    criteria.insert(
        NO_TOOL.to_string(),
        Some("Reply in plain text; no tool call is needed.".into()),
    );

    let mut questions = BTreeMap::new();
    questions.insert(
        "tool".into(),
        Question {
            kind: "choice",
            instructions: "Which tool should the coding agent call next? Pick __no_tool__ if the model should answer in text.".into(),
            criteria: Some(criteria),
        },
    );
    questions.insert(
        "needs_tool".into(),
        Question {
            kind: "noul",
            instructions: "Does the latest user turn require a tool call rather than a text reply?".into(),
            criteria: None,
        },
    );

    let state = json!({
        "body": format!(
            "system:\n{}\n\nuser:\n{}\n\ntools:\n{}",
            truncate(&input.system, 2000),
            truncate(&input.last_user, 4000),
            input
                .tools
                .iter()
                .map(|t| format!("- {}: {}", t.name, truncate(&t.description, 200)))
                .collect::<Vec<_>>()
                .join("\n")
        )
    });

    SystemOneRequest {
        state,
        questions,
        model,
    }
}

pub fn interpret(
    answers: &BTreeMap<String, Answer>,
    min_confidence: f64,
) -> Decision {
    let tool_answer = answers.get("tool");
    let needs = answers.get("needs_tool").and_then(|a| a.noul).unwrap_or(0.5);

    let Some(answer) = tool_answer else {
        return Decision::Passthrough {
            reason: "laya_missing_tool_answer".into(),
        };
    };

    let confidence = answer_confidence(answer);
    let choice = answer.choice.clone().unwrap_or_default();

    if confidence < min_confidence {
        return Decision::Passthrough {
            reason: format!("low_confidence:{confidence:.3}"),
        };
    }

    if choice.is_empty() || choice == NO_TOOL || needs < 0.45 {
        return Decision::None { confidence };
    }

    Decision::Forced {
        tool: choice,
        confidence,
    }
}

fn answer_confidence(answer: &Answer) -> f64 {
    if let Some(c) = answer.confidence {
        return c;
    }
    answer
        .probabilities
        .as_ref()
        .and_then(|p| p.values().cloned().reduce(f64::max))
        .unwrap_or(0.0)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max).collect::<String>() + "…"
}

/// Extract OpenAI-style tools + last user text from a chat completions body.
pub fn extract_from_chat(body: &Value) -> DecideInput {
    let messages = body
        .get("messages")
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default();

    let mut system = String::new();
    let mut last_user = String::new();
    for msg in &messages {
        let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
        let text = message_text(msg.get("content"));
        match role {
            "system" | "developer" => {
                if !system.is_empty() {
                    system.push_str("\n\n");
                }
                system.push_str(&text);
            }
            "user" => last_user = text,
            _ => {}
        }
    }

    let tools = body
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|tool| {
                    if tool.get("type").and_then(|t| t.as_str()) != Some("function") {
                        return None;
                    }
                    let f = tool.get("function")?;
                    let name = f.get("name")?.as_str()?.to_string();
                    let description = f
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();
                    Some(ToolInfo { name, description })
                })
                .collect()
        })
        .unwrap_or_default();

    let tool_choice_decided = match body.get("tool_choice") {
        None => false,
        Some(Value::String(s)) => s != "auto" && s != "required",
        Some(Value::Object(_)) => true,
        _ => false,
    };

    DecideInput {
        system,
        last_user,
        tools,
        tool_choice_decided,
    }
}

fn message_text(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(parts)) => parts
            .iter()
            .filter_map(|p| {
                if p.get("type").and_then(|t| t.as_str()) == Some("text") {
                    p.get("text").and_then(|t| t.as_str()).map(str::to_string)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_when_no_tools() {
        let input = DecideInput {
            system: String::new(),
            last_user: "hi".into(),
            tools: vec![],
            tool_choice_decided: false,
        };
        assert_eq!(skip_reason(&input), Some("no_tools"));
    }

    #[test]
    fn force_when_confident_tool() {
        let mut answers = BTreeMap::new();
        answers.insert(
            "tool".into(),
            Answer {
                kind: "choice".into(),
                choice: Some("read_file".into()),
                confidence: Some(0.91),
                probabilities: None,
                noul: None,
            },
        );
        answers.insert(
            "needs_tool".into(),
            Answer {
                kind: "noul".into(),
                choice: None,
                confidence: None,
                probabilities: None,
                noul: Some(0.88),
            },
        );
        match interpret(&answers, 0.55) {
            Decision::Forced { tool, .. } => assert_eq!(tool, "read_file"),
            other => panic!("expected Forced, got {other:?}"),
        }
    }

    #[test]
    fn passthrough_on_low_confidence() {
        let mut answers = BTreeMap::new();
        answers.insert(
            "tool".into(),
            Answer {
                kind: "choice".into(),
                choice: Some("bash".into()),
                confidence: Some(0.2),
                probabilities: None,
                noul: None,
            },
        );
        match interpret(&answers, 0.55) {
            Decision::Passthrough { reason } => assert!(reason.contains("low_confidence")),
            other => panic!("expected Passthrough, got {other:?}"),
        }
    }
}
