//! Sentiment Analysis
//!
//! Basic sentiment analysis for conversation tracking.
//! Uses keyword-based analysis for speed.

use std::collections::HashSet;

/// Sentiment classification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sentiment {
    VeryPositive,
    Positive,
    Neutral,
    Negative,
    VeryNegative,
}

impl Sentiment {
    pub fn score(&self) -> i32 {
        match self {
            Sentiment::VeryPositive => 2,
            Sentiment::Positive => 1,
            Sentiment::Neutral => 0,
            Sentiment::Negative => -1,
            Sentiment::VeryNegative => -2,
        }
    }

    pub fn from_score(score: f32) -> Self {
        if score > 1.5 {
            Sentiment::VeryPositive
        } else if score > 0.5 {
            Sentiment::Positive
        } else if score > -0.5 {
            Sentiment::Neutral
        } else if score > -1.5 {
            Sentiment::Negative
        } else {
            Sentiment::VeryNegative
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Sentiment::VeryPositive => "Very Positive",
            Sentiment::Positive => "Positive",
            Sentiment::Neutral => "Neutral",
            Sentiment::Negative => "Negative",
            Sentiment::VeryNegative => "Very Negative",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Sentiment::VeryPositive => "😄",
            Sentiment::Positive => "🙂",
            Sentiment::Neutral => "😐",
            Sentiment::Negative => "🙁",
            Sentiment::VeryNegative => "😞",
        }
    }
}

impl Default for Sentiment {
    fn default() -> Self {
        Sentiment::Neutral
    }
}

/// Simple keyword-based sentiment analyzer
pub struct SentimentAnalyzer;

impl SentimentAnalyzer {
    /// Analyze sentiment of text
    pub fn analyze(text: &str) -> Sentiment {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();

        let mut score: f32 = 0.0;
        let mut word_count = 0;

        for word in &words {
            if let Some(s) = Self::word_sentiment(word) {
                score += s;
                word_count += 1;
            }
        }

        // Check for intensifiers and negations
        for i in 0..words.len() {
            if Self::is_negation(words[i]) {
                // Flip sentiment of next word
                if i + 1 < words.len() {
                    if let Some(s) = Self::word_sentiment(words[i + 1]) {
                        score -= s * 2.0; // Undo positive and make negative (or vice versa)
                    }
                }
            }

            if Self::is_intensifier(words[i]) {
                // Amplify sentiment of next word
                if i + 1 < words.len() {
                    if let Some(s) = Self::word_sentiment(words[i + 1]) {
                        score += s * 0.5;
                    }
                }
            }
        }

        // Normalize by word count
        if word_count > 0 {
            score /= word_count as f32;
        }

        Sentiment::from_score(score)
    }

    fn word_sentiment(word: &str) -> Option<f32> {
        // Positive words
        let positive: HashSet<&str> = [
            "good", "great", "excellent", "amazing", "wonderful", "fantastic",
            "perfect", "love", "loved", "loving", "like", "liked", "best",
            "awesome", "brilliant", "outstanding", "happy", "pleased", "glad",
            "excited", "thrilled", "delighted", "satisfied", "impressive",
            "beautiful", "nice", "helpful", "useful", "valuable", "effective",
            "efficient", "successful", "positive", "agree", "yes", "absolutely",
            "definitely", "certainly", "sure", "interested", "exciting",
            "opportunity", "benefit", "advantage", "solution", "solve",
            "improve", "improved", "growth", "grow", "progress", "achieve",
            "accomplished", "win", "won", "winning", "success", "recommend",
        ].into_iter().collect();

        // Very positive words
        let very_positive: HashSet<&str> = [
            "amazing", "incredible", "extraordinary", "exceptional", "phenomenal",
            "outstanding", "remarkable", "superb", "magnificent", "brilliant",
        ].into_iter().collect();

        // Negative words
        let negative: HashSet<&str> = [
            "bad", "poor", "terrible", "awful", "horrible", "worst",
            "hate", "hated", "hating", "dislike", "disappointing", "disappointed",
            "frustrating", "frustrated", "annoying", "annoyed", "angry", "upset",
            "unhappy", "sad", "worried", "concern", "concerned", "problem",
            "issue", "difficult", "hard", "challenging", "struggle", "struggling",
            "fail", "failed", "failure", "wrong", "mistake", "error", "bug",
            "broken", "slow", "expensive", "complicated", "confusing",
            "unclear", "no", "not", "never", "unfortunately", "sorry",
            "regret", "unable", "cannot", "can't", "won't", "doesn't",
        ].into_iter().collect();

        // Very negative words
        let very_negative: HashSet<&str> = [
            "terrible", "horrible", "awful", "disaster", "catastrophe",
            "unacceptable", "outrageous", "furious", "livid",
        ].into_iter().collect();

        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());

        if very_positive.contains(clean_word) {
            Some(2.0)
        } else if positive.contains(clean_word) {
            Some(1.0)
        } else if very_negative.contains(clean_word) {
            Some(-2.0)
        } else if negative.contains(clean_word) {
            Some(-1.0)
        } else {
            None
        }
    }

    fn is_negation(word: &str) -> bool {
        let negations: HashSet<&str> = [
            "not", "no", "never", "neither", "nobody", "nothing",
            "nowhere", "don't", "doesn't", "didn't", "won't", "wouldn't",
            "couldn't", "shouldn't", "can't", "cannot", "isn't", "aren't",
            "wasn't", "weren't", "haven't", "hasn't", "hadn't",
        ].into_iter().collect();

        negations.contains(word.trim_matches(|c: char| !c.is_alphanumeric()))
    }

    fn is_intensifier(word: &str) -> bool {
        let intensifiers: HashSet<&str> = [
            "very", "really", "extremely", "incredibly", "absolutely",
            "completely", "totally", "utterly", "highly", "deeply",
            "so", "such", "quite", "rather",
        ].into_iter().collect();

        intensifiers.contains(word.trim_matches(|c: char| !c.is_alphanumeric()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_sentiment() {
        let sentiment = SentimentAnalyzer::analyze("This is great, I love it!");
        assert!(matches!(sentiment, Sentiment::Positive | Sentiment::VeryPositive));
    }

    #[test]
    fn test_negative_sentiment() {
        let sentiment = SentimentAnalyzer::analyze("This is terrible, I hate it.");
        assert!(matches!(sentiment, Sentiment::Negative | Sentiment::VeryNegative));
    }

    #[test]
    fn test_neutral_sentiment() {
        let sentiment = SentimentAnalyzer::analyze("The meeting is at 3pm.");
        assert!(matches!(sentiment, Sentiment::Neutral));
    }

    #[test]
    fn test_negation() {
        let sentiment = SentimentAnalyzer::analyze("This is not good.");
        assert!(matches!(sentiment, Sentiment::Negative | Sentiment::Neutral));
    }
}
