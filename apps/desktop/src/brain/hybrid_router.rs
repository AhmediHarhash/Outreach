//! Hybrid AI Router
//!
//! Intelligently routes requests between local and cloud AI:
//! - Local LLM (Ollama): Fast responses, simple queries
//! - Cloud AI (OpenAI/Anthropic/Gemini): Complex reasoning, nuanced responses
//!
//! Provides the best of both worlds:
//! - Speed when you need it (local)
//! - Quality when you need it (cloud)
//! - Cost savings by using local when appropriate

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::flash::{FlashAnalysis, GeminiFlash, GPT4oMini, OllamaFlash};

/// Query complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Complexity {
    /// Simple factual or procedural (use local)
    Simple,
    /// Moderate - some nuance needed
    Moderate,
    /// Complex - needs strong reasoning (use cloud)
    Complex,
    /// Critical - high stakes, must be accurate
    Critical,
}

impl Complexity {
    pub fn from_text(text: &str) -> Self {
        let text_lower = text.to_lowercase();
        let word_count = text.split_whitespace().count();

        // Indicators of complexity
        let complex_keywords = [
            "why", "explain", "compare", "analyze", "evaluate", "justify",
            "critique", "strategy", "negotiate", "convince", "objection",
            "budget", "decision", "stakeholder", "executive", "contract",
            "legal", "compliance", "security", "architecture", "scale",
        ];

        let simple_keywords = [
            "what is", "how do", "when", "where", "who", "list",
            "define", "describe", "tell me about", "features",
        ];

        let mut complexity_score = 0;

        // Check for complex patterns
        for keyword in &complex_keywords {
            if text_lower.contains(keyword) {
                complexity_score += 2;
            }
        }

        // Check for simple patterns
        for keyword in &simple_keywords {
            if text_lower.contains(keyword) {
                complexity_score -= 1;
            }
        }

        // Long queries tend to be more complex
        if word_count > 30 {
            complexity_score += 2;
        } else if word_count > 15 {
            complexity_score += 1;
        }

        // Questions with multiple parts
        let question_marks = text.matches('?').count();
        if question_marks > 1 {
            complexity_score += 1;
        }

        // Map score to complexity
        match complexity_score {
            ..=0 => Complexity::Simple,
            1..=3 => Complexity::Moderate,
            4..=6 => Complexity::Complex,
            _ => Complexity::Critical,
        }
    }
}

/// AI provider selection
#[derive(Debug, Clone, PartialEq)]
pub enum AIProvider {
    /// Local Ollama (Llama, Mistral, etc.)
    Local(String), // model name
    /// OpenAI (GPT-4o, GPT-4o-mini)
    OpenAI(String),
    /// Anthropic (Claude)
    Anthropic(String),
    /// Google (Gemini)
    Google(String),
}

impl AIProvider {
    pub fn name(&self) -> &str {
        match self {
            AIProvider::Local(m) => m,
            AIProvider::OpenAI(m) => m,
            AIProvider::Anthropic(m) => m,
            AIProvider::Google(m) => m,
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, AIProvider::Local(_))
    }
}

/// Routing strategy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum RoutingStrategy {
    /// Always use local (maximum privacy, minimum cost)
    AlwaysLocal,
    /// Always use cloud (maximum quality)
    AlwaysCloud,
    /// Smart routing based on complexity (recommended)
    #[default]
    Smart,
    /// Local first, cloud fallback on error
    LocalWithFallback,
    /// Speed priority (use fastest available)
    SpeedFirst,
    /// Quality priority (always use best model)
    QualityFirst,
}

/// Hybrid router configuration
#[derive(Debug, Clone)]
pub struct HybridRouterConfig {
    /// Routing strategy
    pub strategy: RoutingStrategy,

    /// Local model to use
    pub local_model: String,

    /// Cloud models by provider
    pub openai_model: String,
    pub anthropic_model: String,
    pub google_model: String,

    /// API keys
    pub openai_key: Option<String>,
    pub anthropic_key: Option<String>,
    pub google_key: Option<String>,

    /// Timeout for local (short for fast fallback)
    pub local_timeout: Duration,

    /// Threshold for using cloud (complexity level)
    pub cloud_threshold: Complexity,

    /// Prefer local for these modes
    pub prefer_local_modes: Vec<String>,
}

impl Default for HybridRouterConfig {
    fn default() -> Self {
        Self {
            strategy: RoutingStrategy::Smart,
            local_model: "llama3.1:8b".to_string(),
            openai_model: "gpt-4o-mini".to_string(),
            anthropic_model: "claude-3-5-sonnet-20241022".to_string(),
            google_model: "gemini-2.0-flash-exp".to_string(),
            openai_key: None,
            anthropic_key: None,
            google_key: None,
            local_timeout: Duration::from_secs(5),
            cloud_threshold: Complexity::Moderate,
            prefer_local_modes: vec!["technical".to_string()],
        }
    }
}

/// The hybrid AI router
pub struct HybridRouter {
    config: HybridRouterConfig,
    local_available: bool,
}

impl HybridRouter {
    pub fn new(config: HybridRouterConfig) -> Self {
        Self {
            config,
            local_available: false,
        }
    }

    /// Check if local Ollama is available
    pub async fn check_local(&mut self) -> bool {
        let client = OllamaFlash::new().with_model(&self.config.local_model);
        self.local_available = client.is_available().await;
        self.local_available
    }

    /// Determine which provider to use
    pub fn select_provider(&self, text: &str, mode: &str) -> AIProvider {
        let complexity = Complexity::from_text(text);

        match self.config.strategy {
            RoutingStrategy::AlwaysLocal => {
                AIProvider::Local(self.config.local_model.clone())
            }

            RoutingStrategy::AlwaysCloud => {
                self.best_cloud_provider()
            }

            RoutingStrategy::Smart => {
                // Use local for simple/moderate, cloud for complex/critical
                if complexity < self.config.cloud_threshold {
                    if self.local_available {
                        AIProvider::Local(self.config.local_model.clone())
                    } else {
                        self.best_cloud_provider()
                    }
                } else {
                    self.best_cloud_provider()
                }
            }

            RoutingStrategy::LocalWithFallback => {
                if self.local_available {
                    AIProvider::Local(self.config.local_model.clone())
                } else {
                    self.best_cloud_provider()
                }
            }

            RoutingStrategy::SpeedFirst => {
                // Local is usually faster
                if self.local_available {
                    AIProvider::Local(self.config.local_model.clone())
                } else if self.config.google_key.is_some() {
                    // Gemini Flash is fast
                    AIProvider::Google(self.config.google_model.clone())
                } else if self.config.openai_key.is_some() {
                    AIProvider::OpenAI("gpt-4o-mini".to_string())
                } else {
                    AIProvider::Anthropic(self.config.anthropic_model.clone())
                }
            }

            RoutingStrategy::QualityFirst => {
                // Claude > GPT-4o > Gemini > Local
                if self.config.anthropic_key.is_some() {
                    AIProvider::Anthropic(self.config.anthropic_model.clone())
                } else if self.config.openai_key.is_some() {
                    AIProvider::OpenAI("gpt-4o".to_string())
                } else if self.config.google_key.is_some() {
                    AIProvider::Google(self.config.google_model.clone())
                } else {
                    AIProvider::Local(self.config.local_model.clone())
                }
            }
        }
    }

    /// Get best available cloud provider
    fn best_cloud_provider(&self) -> AIProvider {
        // Prefer Gemini Flash for speed, Claude for quality
        if self.config.google_key.is_some() {
            AIProvider::Google(self.config.google_model.clone())
        } else if self.config.openai_key.is_some() {
            AIProvider::OpenAI(self.config.openai_model.clone())
        } else if self.config.anthropic_key.is_some() {
            AIProvider::Anthropic(self.config.anthropic_model.clone())
        } else {
            // Fallback to local if no cloud keys
            AIProvider::Local(self.config.local_model.clone())
        }
    }

    /// Run flash analysis with hybrid routing
    pub async fn analyze_flash(&self, transcript: &str, context: &str) -> Result<(FlashAnalysis, AIProvider)> {
        let provider = self.select_provider(transcript, context);

        tracing::info!("Routing to {:?} (complexity: {:?})",
            provider.name(),
            Complexity::from_text(transcript)
        );

        let result = match &provider {
            AIProvider::Local(model) => {
                let client = OllamaFlash::new().with_model(model.clone());
                client.analyze(transcript, context).await
            }
            AIProvider::Google(model) => {
                let key = self.config.google_key.as_ref().ok_or_else(|| anyhow::anyhow!("No Google key"))?;
                let client = GeminiFlash::new(key.clone()).with_model(model.clone());
                client.analyze(transcript, context).await
            }
            AIProvider::OpenAI(_model) => {
                let key = self.config.openai_key.as_ref().ok_or_else(|| anyhow::anyhow!("No OpenAI key"))?;
                let client = GPT4oMini::new(key.clone());
                client.analyze(transcript, context).await
            }
            AIProvider::Anthropic(_model) => {
                // Use OpenAI as fallback since we don't have Anthropic flash implementation
                let key = self.config.openai_key.as_ref()
                    .or(self.config.google_key.as_ref())
                    .ok_or_else(|| anyhow::anyhow!("No fallback key"))?;

                if self.config.google_key.is_some() {
                    let client = GeminiFlash::new(key.clone());
                    client.analyze(transcript, context).await
                } else {
                    let client = GPT4oMini::new(key.clone());
                    client.analyze(transcript, context).await
                }
            }
        };

        // On error with local, try cloud fallback
        if result.is_err() && provider.is_local() && matches!(self.config.strategy, RoutingStrategy::LocalWithFallback | RoutingStrategy::Smart) {
            tracing::warn!("Local failed, falling back to cloud");
            let cloud_provider = self.best_cloud_provider();
            if !cloud_provider.is_local() {
                return self.analyze_with_provider(transcript, context, &cloud_provider).await;
            }
        }

        Ok((result?, provider))
    }

    /// Analyze with specific provider
    async fn analyze_with_provider(&self, transcript: &str, context: &str, provider: &AIProvider) -> Result<(FlashAnalysis, AIProvider)> {
        let result = match provider {
            AIProvider::Google(model) => {
                let key = self.config.google_key.as_ref().ok_or_else(|| anyhow::anyhow!("No Google key"))?;
                let client = GeminiFlash::new(key.clone()).with_model(model.clone());
                client.analyze(transcript, context).await?
            }
            AIProvider::OpenAI(_) => {
                let key = self.config.openai_key.as_ref().ok_or_else(|| anyhow::anyhow!("No OpenAI key"))?;
                let client = GPT4oMini::new(key.clone());
                client.analyze(transcript, context).await?
            }
            _ => return Err(anyhow::anyhow!("Provider not supported for fallback")),
        };

        Ok((result, provider.clone()))
    }

    /// Get routing explanation for UI
    pub fn explain_routing(&self, text: &str) -> RoutingExplanation {
        let complexity = Complexity::from_text(text);
        let provider = self.select_provider(text, "");

        RoutingExplanation {
            complexity,
            provider_name: provider.name().to_string(),
            is_local: provider.is_local(),
            reason: match (&self.config.strategy, &complexity) {
                (RoutingStrategy::AlwaysLocal, _) => "Using local (always local mode)".to_string(),
                (RoutingStrategy::AlwaysCloud, _) => "Using cloud (always cloud mode)".to_string(),
                (RoutingStrategy::Smart, Complexity::Simple) => "Using local (simple query)".to_string(),
                (RoutingStrategy::Smart, Complexity::Moderate) => "Using local (moderate complexity)".to_string(),
                (RoutingStrategy::Smart, Complexity::Complex) => "Using cloud (complex reasoning needed)".to_string(),
                (RoutingStrategy::Smart, Complexity::Critical) => "Using cloud (critical - high accuracy)".to_string(),
                (RoutingStrategy::SpeedFirst, _) => "Using fastest available".to_string(),
                (RoutingStrategy::QualityFirst, _) => "Using highest quality".to_string(),
                _ => "Auto-selected".to_string(),
            },
        }
    }
}

/// Routing explanation for UI feedback
#[derive(Debug, Clone)]
pub struct RoutingExplanation {
    pub complexity: Complexity,
    pub provider_name: String,
    pub is_local: bool,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_simple() {
        let text = "What is the price?";
        assert_eq!(Complexity::from_text(text), Complexity::Simple);
    }

    #[test]
    fn test_complexity_complex() {
        let text = "Can you explain why your enterprise solution would be better for our compliance and security requirements compared to the competition, and how would you justify the budget to our stakeholders?";
        let complexity = Complexity::from_text(text);
        assert!(complexity >= Complexity::Complex);
    }

    #[test]
    fn test_smart_routing() {
        let config = HybridRouterConfig {
            openai_key: Some("test".to_string()),
            ..Default::default()
        };
        let mut router = HybridRouter::new(config);
        router.local_available = true;

        // Simple should go local
        let provider = router.select_provider("What is the price?", "sales");
        assert!(provider.is_local());

        // Complex should go cloud
        let provider = router.select_provider(
            "Explain why this architecture would scale better and justify the budget",
            "sales"
        );
        assert!(!provider.is_local());
    }
}
