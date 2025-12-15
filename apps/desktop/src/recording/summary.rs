//! Call Summary & Self-Analysis
//!
//! AI-powered analysis of your conversation:
//! - What the caller needed
//! - What you did well
//! - What could have been better
//! - How you delivered the AI suggestions
//! - Overall performance score

use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::session::RecordingSession;

/// Complete call summary with self-analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSummary {
    /// Session ID this summary is for
    pub session_id: String,

    /// When the summary was generated
    pub generated_at: DateTime<Utc>,

    /// Overall performance score (0-100)
    pub score: PerformanceScore,

    /// What the caller/other person needed
    pub caller_needs: Vec<String>,

    /// What you did during the call
    pub what_you_did: Vec<String>,

    /// What you did well
    pub did_well: Vec<String>,

    /// What could have been better
    pub could_improve: Vec<String>,

    /// Alternative approaches that might have worked better
    pub alternative_approaches: Vec<String>,

    /// How you delivered the AI suggestions
    pub delivery_analysis: DeliveryAnalysis,

    /// Outcome assessment
    pub outcome: OutcomeAssessment,

    /// Key moments in the conversation
    pub key_moments: Vec<KeyMoment>,

    /// Actionable next steps
    pub next_steps: Vec<String>,

    /// One-paragraph executive summary
    pub executive_summary: String,
}

/// Performance score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceScore {
    /// Overall score (0-100)
    pub overall: u32,

    /// Listening score - did you understand their needs?
    pub listening: u32,

    /// Response quality - were your responses appropriate?
    pub response_quality: u32,

    /// Delivery - how well did you speak?
    pub delivery: u32,

    /// Suggestion usage - did you use the AI help effectively?
    pub suggestion_usage: u32,

    /// Outcome - did you achieve the goal?
    pub outcome: u32,

    /// Grade (A, B, C, D, F)
    pub grade: String,

    /// One-line assessment
    pub assessment: String,
}

impl PerformanceScore {
    pub fn calculate(
        listening: u32,
        response_quality: u32,
        delivery: u32,
        suggestion_usage: u32,
        outcome: u32,
    ) -> Self {
        let overall = (listening + response_quality + delivery + suggestion_usage + outcome) / 5;

        let grade = match overall {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        }.to_string();

        let assessment = match overall {
            90..=100 => "Excellent performance! You nailed it.",
            80..=89 => "Great job! Minor improvements possible.",
            70..=79 => "Good effort. Some areas need work.",
            60..=69 => "Acceptable but significant room for improvement.",
            _ => "Needs improvement. Review the suggestions.",
        }.to_string();

        Self {
            overall,
            listening,
            response_quality,
            delivery,
            suggestion_usage,
            outcome,
            grade,
            assessment,
        }
    }
}

/// Analysis of how you delivered the AI suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryAnalysis {
    /// Speaking pace assessment
    pub pace: PaceAssessment,

    /// Clarity of speech
    pub clarity: ClarityAssessment,

    /// Natural vs robotic
    pub naturalness: u32, // 0-100

    /// Confidence level
    pub confidence: u32, // 0-100

    /// Did you personalize the suggestions or read verbatim?
    pub personalization: String,

    /// Specific feedback on delivery
    pub feedback: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaceAssessment {
    TooFast,
    SlightlyFast,
    Perfect,
    SlightlySlow,
    TooSlow,
}

impl PaceAssessment {
    pub fn label(&self) -> &'static str {
        match self {
            Self::TooFast => "Too fast - slow down",
            Self::SlightlyFast => "Slightly fast",
            Self::Perfect => "Perfect pace",
            Self::SlightlySlow => "Slightly slow",
            Self::TooSlow => "Too slow - pick up the pace",
        }
    }

    pub fn from_wpm(wpm: f32) -> Self {
        match wpm as u32 {
            0..=100 => Self::TooSlow,
            101..=120 => Self::SlightlySlow,
            121..=160 => Self::Perfect,
            161..=180 => Self::SlightlyFast,
            _ => Self::TooFast,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClarityAssessment {
    VeryUnclear,
    Unclear,
    Acceptable,
    Clear,
    VeryClear,
}

/// Outcome assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeAssessment {
    /// Did you achieve the goal?
    pub goal_achieved: GoalStatus,

    /// What was the likely outcome?
    pub likely_outcome: String,

    /// How close were you to success? (0-100%)
    pub success_proximity: u32,

    /// What would have made the difference?
    pub difference_maker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalStatus {
    Achieved,
    PartiallyAchieved,
    NotAchieved,
    TooEarlyToTell,
}

impl GoalStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Achieved => "Goal achieved!",
            Self::PartiallyAchieved => "Partially achieved",
            Self::NotAchieved => "Goal not achieved",
            Self::TooEarlyToTell => "Too early to tell",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Achieved => "✅",
            Self::PartiallyAchieved => "🔶",
            Self::NotAchieved => "❌",
            Self::TooEarlyToTell => "⏳",
        }
    }
}

/// Key moment in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMoment {
    /// When it happened
    pub timestamp: DateTime<Utc>,

    /// What was said
    pub quote: String,

    /// Why it was significant
    pub significance: String,

    /// Was it positive or negative?
    pub sentiment: MomentSentiment,

    /// What you should have done (if different)
    pub ideal_response: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MomentSentiment {
    Positive,
    Neutral,
    Negative,
    Critical,
}

/// Self-analysis breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAnalysis {
    /// Strengths demonstrated
    pub strengths: Vec<String>,

    /// Weaknesses observed
    pub weaknesses: Vec<String>,

    /// Patterns noticed
    pub patterns: Vec<String>,

    /// Recommendations for improvement
    pub recommendations: Vec<String>,

    /// Comparison to ideal performance
    pub ideal_comparison: String,
}

/// Generate a comprehensive call summary using AI
pub async fn generate_call_summary(
    session: &RecordingSession,
    api_key: &str,
    model: &str,
) -> Result<CallSummary> {
    let transcript = session.full_transcript();
    let mode = &session.mode;
    let duration = session.duration();

    // Build the analysis prompt
    let prompt = format!(
        r#"Analyze this {mode} conversation and provide a comprehensive assessment.

TRANSCRIPT:
{transcript}

SESSION INFO:
- Duration: {duration_mins} minutes
- Mode: {mode}
- User talk time: {user_pct}%
- Other person talk time: {other_pct}%
- AI suggestions provided: {suggestions}
- Suggestions used: {used}

Provide analysis in the following JSON format:
{{
    "caller_needs": ["what they needed/wanted"],
    "what_you_did": ["actions you took"],
    "did_well": ["things done well"],
    "could_improve": ["areas for improvement"],
    "alternative_approaches": ["what might have worked better"],
    "delivery": {{
        "pace": "too_fast|slightly_fast|perfect|slightly_slow|too_slow",
        "naturalness": 0-100,
        "confidence": 0-100,
        "feedback": ["specific delivery feedback"]
    }},
    "outcome": {{
        "status": "achieved|partial|not_achieved|unknown",
        "proximity": 0-100,
        "difference_maker": "what would have made the difference"
    }},
    "key_moments": [
        {{
            "quote": "what was said",
            "significance": "why it mattered",
            "sentiment": "positive|neutral|negative|critical",
            "ideal_response": "what you should have said (if different)"
        }}
    ],
    "scores": {{
        "listening": 0-100,
        "response_quality": 0-100,
        "delivery": 0-100,
        "suggestion_usage": 0-100,
        "outcome": 0-100
    }},
    "next_steps": ["actionable follow-ups"],
    "executive_summary": "one paragraph summary"
}}

Be honest and constructive. Focus on actionable insights."#,
        mode = mode,
        transcript = transcript,
        duration_mins = duration.num_minutes(),
        user_pct = (session.talk_ratio() * 100.0) as u32,
        other_pct = ((1.0 - session.talk_ratio()) * 100.0) as u32,
        suggestions = session.metadata.total_suggestions,
        used = session.metadata.suggestions_used,
    );

    // Call the AI API (using OpenAI format)
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are an expert conversation analyst and coach. Analyze conversations and provide actionable, honest feedback."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7,
            "max_tokens": 2000,
            "response_format": { "type": "json_object" }
        }))
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;

    // Parse the response
    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

    let analysis: serde_json::Value = serde_json::from_str(content)?;

    // Build the CallSummary
    let scores = &analysis["scores"];
    let score = PerformanceScore::calculate(
        scores["listening"].as_u64().unwrap_or(70) as u32,
        scores["response_quality"].as_u64().unwrap_or(70) as u32,
        scores["delivery"].as_u64().unwrap_or(70) as u32,
        scores["suggestion_usage"].as_u64().unwrap_or(70) as u32,
        scores["outcome"].as_u64().unwrap_or(70) as u32,
    );

    let delivery = &analysis["delivery"];
    let delivery_analysis = DeliveryAnalysis {
        pace: match delivery["pace"].as_str().unwrap_or("perfect") {
            "too_fast" => PaceAssessment::TooFast,
            "slightly_fast" => PaceAssessment::SlightlyFast,
            "slightly_slow" => PaceAssessment::SlightlySlow,
            "too_slow" => PaceAssessment::TooSlow,
            _ => PaceAssessment::Perfect,
        },
        clarity: ClarityAssessment::Clear,
        naturalness: delivery["naturalness"].as_u64().unwrap_or(70) as u32,
        confidence: delivery["confidence"].as_u64().unwrap_or(70) as u32,
        personalization: "Adapted suggestions to context".to_string(),
        feedback: delivery["feedback"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
    };

    let outcome_data = &analysis["outcome"];
    let outcome = OutcomeAssessment {
        goal_achieved: match outcome_data["status"].as_str().unwrap_or("unknown") {
            "achieved" => GoalStatus::Achieved,
            "partial" => GoalStatus::PartiallyAchieved,
            "not_achieved" => GoalStatus::NotAchieved,
            _ => GoalStatus::TooEarlyToTell,
        },
        likely_outcome: "Based on conversation trajectory".to_string(),
        success_proximity: outcome_data["proximity"].as_u64().unwrap_or(50) as u32,
        difference_maker: outcome_data["difference_maker"].as_str().map(String::from),
    };

    let key_moments: Vec<KeyMoment> = analysis["key_moments"]
        .as_array()
        .map(|moments| {
            moments
                .iter()
                .map(|m| KeyMoment {
                    timestamp: Utc::now(), // Would need actual timestamps
                    quote: m["quote"].as_str().unwrap_or("").to_string(),
                    significance: m["significance"].as_str().unwrap_or("").to_string(),
                    sentiment: match m["sentiment"].as_str().unwrap_or("neutral") {
                        "positive" => MomentSentiment::Positive,
                        "negative" => MomentSentiment::Negative,
                        "critical" => MomentSentiment::Critical,
                        _ => MomentSentiment::Neutral,
                    },
                    ideal_response: m["ideal_response"].as_str().map(String::from),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(CallSummary {
        session_id: session.id.clone(),
        generated_at: Utc::now(),
        score,
        caller_needs: extract_string_array(&analysis["caller_needs"]),
        what_you_did: extract_string_array(&analysis["what_you_did"]),
        did_well: extract_string_array(&analysis["did_well"]),
        could_improve: extract_string_array(&analysis["could_improve"]),
        alternative_approaches: extract_string_array(&analysis["alternative_approaches"]),
        delivery_analysis,
        outcome,
        key_moments,
        next_steps: extract_string_array(&analysis["next_steps"]),
        executive_summary: analysis["executive_summary"]
            .as_str()
            .unwrap_or("Summary not available")
            .to_string(),
    })
}

fn extract_string_array(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

/// Generate a quick summary without AI (local analysis)
pub fn generate_quick_summary(session: &RecordingSession) -> CallSummary {
    let duration = session.duration();
    let talk_ratio = session.talk_ratio();

    // Calculate basic scores
    let listening = if talk_ratio < 0.5 { 80 } else { 60 }; // Less talking = better listening
    let suggestion_usage = (session.metadata.suggestion_usage_rate() * 100.0) as u32;

    let score = PerformanceScore::calculate(
        listening,
        70, // Default
        70, // Default
        suggestion_usage,
        70, // Default
    );

    let pace = PaceAssessment::from_wpm(session.metadata.user_wpm());

    CallSummary {
        session_id: session.id.clone(),
        generated_at: Utc::now(),
        score,
        caller_needs: vec!["Analysis requires AI processing".to_string()],
        what_you_did: vec![
            format!("Spoke for {}% of the call", (talk_ratio * 100.0) as u32),
            format!("Used {} of {} suggestions", session.metadata.suggestions_used, session.metadata.total_suggestions),
        ],
        did_well: vec!["Quick summary - enable AI for detailed analysis".to_string()],
        could_improve: vec!["Enable AI summary for specific feedback".to_string()],
        alternative_approaches: vec![],
        delivery_analysis: DeliveryAnalysis {
            pace,
            clarity: ClarityAssessment::Acceptable,
            naturalness: 70,
            confidence: 70,
            personalization: "Unknown".to_string(),
            feedback: vec![],
        },
        outcome: OutcomeAssessment {
            goal_achieved: GoalStatus::TooEarlyToTell,
            likely_outcome: "Enable AI for outcome analysis".to_string(),
            success_proximity: 50,
            difference_maker: None,
        },
        key_moments: vec![],
        next_steps: vec!["Review the full transcript".to_string()],
        executive_summary: format!(
            "Call lasted {} minutes. You spoke {}% of the time at {} WPM. Used {}/{} AI suggestions.",
            duration.num_minutes(),
            (talk_ratio * 100.0) as u32,
            session.metadata.user_wpm() as u32,
            session.metadata.suggestions_used,
            session.metadata.total_suggestions
        ),
    }
}
