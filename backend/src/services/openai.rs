use anyhow::Context;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::models::{RewriteStyle, RewriteTone};

#[derive(Clone)]
pub struct OpenAIClient {
    client: Client,
    api_key: String,
    base_url: String,
}

// ── Structured Output Schemas ─────────────────────────────────────────────────

/// The structured response returned by OpenAI for a rewrite
#[derive(Debug, Serialize, Deserialize)]
pub struct RewriteResponse {
    pub rewritten_text: String,
    pub changes_summary: String,
    pub readability_score: u8,    // 1-10
    pub tone_match_score: u8,     // 1-10
    pub word_count_original: u32,
    pub word_count_rewritten: u32,
    pub improvements: Vec<Improvement>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Improvement {
    pub category: ImprovementCategory,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImprovementCategory {
    Clarity,
    Grammar,
    Flow,
    Vocabulary,
    Structure,
    Tone,
}

/// Text analysis before rewriting
#[derive(Debug, Serialize, Deserialize)]
pub struct TextAnalysis {
    pub overall_quality: u8,        // 1-10
    pub readability_level: String,  // e.g. "8th grade"
    pub tone_detected: String,
    pub issues: Vec<TextIssue>,
    pub strengths: Vec<String>,
    pub suggested_tone: String,
    pub word_count: u32,
    pub avg_sentence_length: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextIssue {
    pub severity: IssueSeverity,
    pub category: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
}

// ── OpenAI JSON Schema helpers ────────────────────────────────────────────────

fn rewrite_response_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "rewritten_text": { "type": "string" },
            "changes_summary": { "type": "string" },
            "readability_score": { "type": "integer", "minimum": 1, "maximum": 10 },
            "tone_match_score": { "type": "integer", "minimum": 1, "maximum": 10 },
            "word_count_original": { "type": "integer" },
            "word_count_rewritten": { "type": "integer" },
            "improvements": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "category": {
                            "type": "string",
                            "enum": ["clarity", "grammar", "flow", "vocabulary", "structure", "tone"]
                        },
                        "description": { "type": "string" }
                    },
                    "required": ["category", "description"],
                    "additionalProperties": false
                }
            },
            "warnings": { "type": "array", "items": { "type": "string" } }
        },
        "required": [
            "rewritten_text", "changes_summary", "readability_score",
            "tone_match_score", "word_count_original", "word_count_rewritten",
            "improvements", "warnings"
        ],
        "additionalProperties": false
    })
}

fn text_analysis_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "overall_quality": { "type": "integer", "minimum": 1, "maximum": 10 },
            "readability_level": { "type": "string" },
            "tone_detected": { "type": "string" },
            "issues": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "severity": { "type": "string", "enum": ["low", "medium", "high"] },
                        "category": { "type": "string" },
                        "description": { "type": "string" }
                    },
                    "required": ["severity", "category", "description"],
                    "additionalProperties": false
                }
            },
            "strengths": { "type": "array", "items": { "type": "string" } },
            "suggested_tone": { "type": "string" },
            "word_count": { "type": "integer" },
            "avg_sentence_length": { "type": "number" }
        },
        "required": [
            "overall_quality", "readability_level", "tone_detected",
            "issues", "strengths", "suggested_tone", "word_count", "avg_sentence_length"
        ],
        "additionalProperties": false
    })
}

// ── Client implementation ─────────────────────────────────────────────────────

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.openai.com/v1".into(),
        }
    }

    async fn chat_structured<T: for<'de> Deserialize<'de>>(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        schema_name: &str,
        schema: Value,
    ) -> crate::Result<T> {
        let body = json!({
            "model": "gpt-4o-2024-08-06",
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ],
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": schema_name,
                    "strict": true,
                    "schema": schema
                }
            },
            "temperature": 0.7,
            "max_tokens": 4096
        });

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .context("Failed to call OpenAI")?;

        if !resp.status().is_success() {
            let err: Value = resp.json().await.unwrap_or_default();
            return Err(crate::AppError::OpenAI(
                err["error"]["message"]
                    .as_str()
                    .unwrap_or("Unknown OpenAI error")
                    .to_string(),
            ));
        }

        let data: Value = resp.json().await.context("Failed to parse OpenAI response")?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .context("Missing content in OpenAI response")?;

        // Check for refusal
        if let Some(refusal) = data["choices"][0]["message"]["refusal"].as_str() {
            return Err(crate::AppError::OpenAI(format!("Refused: {}", refusal)));
        }

        serde_json::from_str(content)
            .context("Failed to deserialize structured response")
            .map_err(crate::AppError::Internal)
    }

    /// Analyze text quality before rewriting
    pub async fn analyze_text(&self, text: &str) -> crate::Result<TextAnalysis> {
        let system = "You are an expert writing coach and linguist. \
            Analyze the provided text and return a structured assessment.";

        let user = format!(
            "Analyze this text:\n\n<text>\n{}\n</text>",
            text
        );

        self.chat_structured(
            system,
            &user,
            "text_analysis",
            text_analysis_schema(),
        )
        .await
    }

    /// Rewrite text with structured output
    pub async fn rewrite_text(
        &self,
        original: &str,
        tone: &RewriteTone,
        style: &RewriteStyle,
    ) -> crate::Result<RewriteResponse> {
        let tone_str = format!("{:?}", tone).to_lowercase();
        let style_str = format!("{:?}", style).to_lowercase();

        let system = format!(
            "You are an expert writing assistant specializing in text transformation. \
            Your task is to rewrite text with these parameters:\n\
            - Tone: {tone_str}\n\
            - Style/goal: {style_str}\n\n\
            Preserve the core meaning while applying the requested transformation. \
            Track exactly what you change and why."
        );

        let user = format!(
            "Please rewrite the following text:\n\n<original_text>\n{}\n</original_text>",
            original
        );

        self.chat_structured(
            &system,
            &user,
            "rewrite_response",
            rewrite_response_schema(),
        )
        .await
    }

    /// Auto-rewrite: analyze first, then apply best rewrite automatically
    pub async fn auto_rewrite(&self, text: &str) -> crate::Result<(TextAnalysis, RewriteResponse)> {
        // Step 1: analyze
        let analysis = self.analyze_text(text).await?;

        // Step 2: pick tone & style based on analysis
        let (tone, style) = pick_best_strategy(&analysis);

        // Step 3: rewrite
        let rewrite = self.rewrite_text(text, &tone, &style).await?;

        Ok((analysis, rewrite))
    }
}

/// Heuristic: pick the best rewrite strategy based on analysis
fn pick_best_strategy(analysis: &TextAnalysis) -> (RewriteTone, RewriteStyle) {
    let has_grammar = analysis.issues.iter().any(|i| i.category == "grammar");
    let quality = analysis.overall_quality;

    let style = if has_grammar {
        RewriteStyle::Fix
    } else if quality < 5 {
        RewriteStyle::Improve
    } else if analysis.avg_sentence_length > 25.0 {
        RewriteStyle::Condense
    } else {
        RewriteStyle::Improve
    };

    let tone = match analysis.suggested_tone.to_lowercase().as_str() {
        t if t.contains("casual") => RewriteTone::Casual,
        t if t.contains("academic") => RewriteTone::Academic,
        t if t.contains("persuasive") => RewriteTone::Persuasive,
        t if t.contains("concise") => RewriteTone::Concise,
        _ => RewriteTone::Professional,
    };

    (tone, style)
}
