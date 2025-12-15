//! Conversation Metrics
//!
//! Tracks quantitative metrics about the conversation.

use std::collections::HashMap;

/// Overall conversation metrics
#[derive(Debug, Clone, Default)]
pub struct ConversationMetrics {
    /// User's metrics
    pub user: SpeakerMetrics,
    /// Other speaker's metrics
    pub other: SpeakerMetrics,
}

impl ConversationMetrics {
    /// Get total talk time
    pub fn total_talk_time_ms(&self) -> u64 {
        self.user.total_talk_time_ms + self.other.total_talk_time_ms
    }

    /// Get total word count
    pub fn total_word_count(&self) -> usize {
        self.user.word_count + self.other.word_count
    }

    /// Get total turn count
    pub fn total_turns(&self) -> usize {
        self.user.turn_count + self.other.turn_count
    }
}

/// Metrics for a single speaker
#[derive(Debug, Clone, Default)]
pub struct SpeakerMetrics {
    /// Total time spent talking (milliseconds)
    pub total_talk_time_ms: u64,
    /// Number of conversation turns
    pub turn_count: usize,
    /// Total word count
    pub word_count: usize,
    /// Number of questions asked
    pub question_count: usize,
    /// Longest turn (word count)
    pub longest_turn_words: usize,
    /// Average turn length (words)
    pub avg_turn_words: f32,
}

impl SpeakerMetrics {
    /// Calculate words per minute
    pub fn words_per_minute(&self) -> f32 {
        if self.total_talk_time_ms == 0 {
            return 0.0;
        }
        let minutes = self.total_talk_time_ms as f32 / 60000.0;
        self.word_count as f32 / minutes
    }

    /// Update average turn length
    pub fn update_averages(&mut self) {
        if self.turn_count > 0 {
            self.avg_turn_words = self.word_count as f32 / self.turn_count as f32;
        }
    }
}

/// Topic tracker - extracts and counts key topics
#[derive(Debug, Clone, Default)]
pub struct TopicTracker {
    /// Topic counts
    topics: HashMap<String, usize>,
    /// Keywords that indicate topics
    keywords: Vec<(String, String)>, // (keyword, topic)
}

impl TopicTracker {
    pub fn new() -> Self {
        let mut tracker = Self::default();

        // Add default topic keywords
        tracker.add_keyword_mappings(vec![
            // Sales topics
            ("price", "Pricing"),
            ("pricing", "Pricing"),
            ("cost", "Pricing"),
            ("budget", "Budget"),
            ("discount", "Pricing"),
            ("roi", "ROI"),
            ("return", "ROI"),
            ("investment", "ROI"),
            ("contract", "Contract"),
            ("agreement", "Contract"),
            ("timeline", "Timeline"),
            ("deadline", "Timeline"),
            ("feature", "Features"),
            ("functionality", "Features"),
            ("integration", "Integration"),
            ("api", "Integration"),
            ("support", "Support"),
            ("onboarding", "Onboarding"),
            ("training", "Training"),
            ("security", "Security"),
            ("compliance", "Compliance"),
            ("competitor", "Competition"),
            ("alternative", "Competition"),

            // Interview topics
            ("experience", "Experience"),
            ("skill", "Skills"),
            ("project", "Projects"),
            ("team", "Team"),
            ("leadership", "Leadership"),
            ("challenge", "Challenges"),
            ("problem", "Problem Solving"),
            ("solve", "Problem Solving"),
            ("weakness", "Weaknesses"),
            ("strength", "Strengths"),
            ("goal", "Goals"),
            ("salary", "Compensation"),
            ("compensation", "Compensation"),
            ("benefit", "Benefits"),
            ("culture", "Culture"),
            ("remote", "Remote Work"),

            // Technical topics
            ("performance", "Performance"),
            ("scalability", "Scalability"),
            ("architecture", "Architecture"),
            ("database", "Database"),
            ("deployment", "Deployment"),
            ("testing", "Testing"),
            ("bug", "Bugs"),
            ("error", "Errors"),
            ("documentation", "Documentation"),
        ]);

        tracker
    }

    fn add_keyword_mappings(&mut self, mappings: Vec<(&str, &str)>) {
        for (keyword, topic) in mappings {
            self.keywords.push((keyword.to_lowercase(), topic.to_string()));
        }
    }

    /// Extract topics from text
    pub fn extract_topics(&mut self, text: &str) {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();

        for (keyword, topic) in &self.keywords {
            if words.iter().any(|w| w.contains(keyword.as_str())) {
                *self.topics.entry(topic.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Get top N topics
    pub fn top_topics(&self, n: usize) -> Vec<(&String, usize)> {
        let mut sorted: Vec<_> = self.topics.iter().map(|(k, v)| (k, *v)).collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(n).collect()
    }

    /// Get all topics
    pub fn all_topics(&self) -> &HashMap<String, usize> {
        &self.topics
    }

    /// Clear topics
    pub fn clear(&mut self) {
        self.topics.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_extraction() {
        let mut tracker = TopicTracker::new();

        tracker.extract_topics("What's the price of your enterprise plan?");
        tracker.extract_topics("How does the pricing work for teams?");
        tracker.extract_topics("Tell me about the security features.");

        let topics = tracker.top_topics(3);
        assert!(topics.iter().any(|(t, _)| *t == "Pricing"));

        let pricing_count = tracker.topics.get("Pricing").unwrap_or(&0);
        assert_eq!(*pricing_count, 2);
    }

    #[test]
    fn test_words_per_minute() {
        let mut metrics = SpeakerMetrics {
            total_talk_time_ms: 60000, // 1 minute
            word_count: 150,
            ..Default::default()
        };

        assert!((metrics.words_per_minute() - 150.0).abs() < 0.1);
    }
}
