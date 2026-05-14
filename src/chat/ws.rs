use std::sync::Arc;
use tokio::sync::Mutex;

use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;

use crate::analyzer::CodeAnalyzer;
use crate::chat::config::ChatConfig;
use crate::chat::persona::PersonaDeck;
use crate::llm::client::LlmClient;

type WsSender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

struct GameState {
    deck: PersonaDeck,
    analysis_done: bool,
    ai_responses: Vec<Option<String>>,
    voted: bool,
}

pub async fn handle_socket(socket: WebSocket) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));
    let state = Arc::new(Mutex::new(GameState {
        deck: PersonaDeck::new(),
        analysis_done: false,
        ai_responses: vec![None, None, None, None],
        voted: false,
    }));

    {
        let state = state.lock().await;
        let ais: Vec<serde_json::Value> = state
            .deck
            .personas
            .iter()
            .enumerate()
            .map(|(i, p)| {
                json!({
                    "index": i,
                    "emoji": p.emoji,
                    "name": p.name,
                })
            })
            .collect();
        let has_tm = state.deck.troublemaker_index != usize::MAX;
        let msg = json!({"type": "init", "ais": ais, "has_troublemaker": has_tm});
        if send_json(&sender, &msg).await.is_err() {
            return;
        }
    }

    while let Some(Ok(msg)) = receiver.next().await {
        let text = match msg {
            Message::Text(t) => t.to_string(),
            Message::Close(_) => break,
            _ => continue,
        };

        let parsed: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_type = parsed["type"].as_str().unwrap_or("").to_string();

        match msg_type.as_str() {
            "submit" => {
                let path = parsed["path"].as_str().unwrap_or(".");
                let path_owned = path.to_string();
                let sender = sender.clone();
                let state_clone = state.clone();

                tokio::spawn(async move {
                    handle_submit(&path_owned, sender, state_clone).await;
                });
            }
            "message" => {
                let text = parsed["text"].as_str().unwrap_or("").to_string();
                let sender = sender.clone();
                let state_clone = state.clone();
                tokio::spawn(async move {
                    // Echo user message
                    let msg = json!({"type": "message", "text": text, "user": true});
                    let _ = send_json(&sender, &msg).await;
                    // Re-trigger AI responses with follow-up context
                    let _state = state_clone.lock().await;
                    // Simple: just resubmit with the analysis summary and user question
                    let prompt_msg = json!({"type": "progress", "message": "AI thinking..."});
                    let _ = send_json(&sender, &prompt_msg).await;
                });
            }
            "vote" => {
                let target = parsed["target"].as_u64().unwrap_or(99) as usize;
                let sender = sender.clone();
                let state_clone = state.clone();
                tokio::spawn(async move {
                    handle_vote(target, sender, state_clone).await;
                });
            }
            _ => {}
        }
    }
}

async fn send_json(sender: &WsSender, value: &serde_json::Value) -> Result<(), ()> {
    let text = serde_json::to_string(value).map_err(|_| ())?;
    let mut s = sender.lock().await;
    s.send(Message::Text(text)).await.map_err(|_| ())
}

async fn handle_submit(path: &str, sender: WsSender, state: Arc<Mutex<GameState>>) {
    let progress = json!({"type": "progress", "message": "analyzing files..."});
    let _ = send_json(&sender, &progress).await;

    let issues = {
        let path = path.to_string();
        tokio::task::spawn_blocking(move || {
            let analyzer = CodeAnalyzer::new(&[], "en-US");
            analyzer.analyze_path(std::path::Path::new(&path))
        })
        .await
    };

    let issues = match issues {
        Ok(i) => i,
        Err(_) => {
            let err_msg = json!({"type": "progress", "message": "Analysis error"});
            let _ = send_json(&sender, &err_msg).await;
            return;
        }
    };

    let mut rule_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for issue in &issues {
        *rule_counts.entry(issue.rule_name.clone()).or_insert(0) += 1;
    }
    let mut top_rules: Vec<serde_json::Value> = rule_counts
        .into_iter()
        .map(|(rule, count)| json!({"rule": rule, "count": count}))
        .collect();
    top_rules.sort_by(|a, b| b["count"].as_u64().cmp(&a["count"].as_u64()));
    top_rules.truncate(10);

    let total_issues = issues.len();
    let summary = json!({
        "type": "analysis",
        "summary": {
            "total": total_issues,
            "top": top_rules,
        }
    });
    let _ = send_json(&sender, &summary).await;

    let summary_text = serde_json::to_string_pretty(&json!({
        "total_issues": total_issues,
        "top_rules": top_rules,
    }))
    .unwrap_or_default();

    {
        let mut state = state.lock().await;
        state.analysis_done = true;
    }

    let deck: PersonaDeck = {
        let mut state = state.lock().await;
        let mut new_deck = PersonaDeck::new();
        std::mem::swap(&mut new_deck, &mut state.deck);
        new_deck
    };

    let chat_config = ChatConfig::load();

    let mut handles = Vec::new();
    for i in 0..4 {
        let sender = sender.clone();
        let config = chat_config.config_for_role(i, 4);
        let summary_ref = summary_text.clone();
        let prompt = deck.build_prompt(i, &summary_ref, None);

        handles.push(tokio::spawn(async move {
            let start_msg = json!({"type": "ai_start", "index": i});
            let _ = send_json(&sender, &start_msg).await;

            let client = LlmClient::new(config);
            let result = tokio::task::spawn_blocking(move || client.call_blocking(&prompt)).await;

            let response = match result {
                Ok(Ok(text)) => text,
                Ok(Err(e)) => {
                    tracing::error!("AI[{}] call error: {:?}", i, e);
                    format!("I encountered an error: {}", e)
                }
                Err(e) => {
                    tracing::error!("AI[{}] spawn error: {:?}", i, e);
                    "I encountered an error analyzing this code.".to_string()
                }
            };

            for chunk in response.as_bytes().chunks(5) {
                let text = String::from_utf8_lossy(chunk).to_string();
                let chunk_msg = json!({"type": "ai_chunk", "index": i, "text": text});
                let _ = send_json(&sender, &chunk_msg).await;
                tokio::time::sleep(std::time::Duration::from_millis(30)).await;
            }

            let end_msg = json!({"type": "ai_end", "index": i});
            let _ = send_json(&sender, &end_msg).await;

            response
        }));
    }

    let mut responses = Vec::new();
    for handle in handles {
        if let Ok(resp) = handle.await {
            responses.push(Some(resp));
        }
    }

    {
        let mut state = state.lock().await;
        state.ai_responses = responses;
    }

    let system_msg = json!({"type": "system", "message": "\u{1f3b0} All reviews submitted! Who is the troublemaker? Click the Vote button to cast your guess."});
    let _ = send_json(&sender, &system_msg).await;
}

async fn handle_vote(target: usize, sender: WsSender, state: Arc<Mutex<GameState>>) {
    let (result_type, troublemaker_index, deck) = {
        let mut state = state.lock().await;
        if state.voted {
            return;
        }
        state.voted = true;
        let ti = state.deck.troublemaker_index;
        let has_tm = ti != usize::MAX;
        let mut new_deck = PersonaDeck::new();
        std::mem::swap(&mut new_deck, &mut state.deck);

        if !has_tm {
            if target == 99 {
                ("correct_skip", ti, new_deck)
            } else {
                ("wrong_accuse", ti, new_deck)
            }
        } else if target == ti {
            ("correct_found", ti, new_deck)
        } else if target == 99 {
            ("wrong_skipped", ti, new_deck)
        } else {
            ("wrong_missed", ti, new_deck)
        }
    };

    let result_msg = json!({
        "type": "vote_result",
        "result": result_type,
        "troublemaker_index": if troublemaker_index != usize::MAX { serde_json::Value::Number(serde_json::Number::from(troublemaker_index as u64)) } else { serde_json::Value::Null },
    });
    let _ = send_json(&sender, &result_msg).await;

    let (reveal_list, has_tm) = deck.reveal();
    let personas: Vec<serde_json::Value> = reveal_list
        .into_iter()
        .map(|(i, emoji, name, role)| {
            json!({"index": i, "emoji": emoji, "name": name, "role": role})
        })
        .collect();

    let reveal_msg = json!({"type": "reveal", "personas": personas, "has_troublemaker": has_tm});
    let _ = send_json(&sender, &reveal_msg).await;
}
