# Outreach - Feature Specifications

> Detailed specifications for all features.

---

## 1. Email System

### 1.1 Email Sending

**Single Email:**
- Compose or use template
- AI enhancement option
- Attach CV/documents
- Schedule for later
- Track delivery

**Bulk Email:**
- Select multiple leads
- Template with variables
- Personalization per lead
- Staggered sending (avoid spam)
- Progress tracking

### 1.2 Email Tracking

**Open Tracking:**
- Invisible 1x1 pixel
- Timestamp recorded
- Multiple opens counted
- Device/location (optional)

**Click Tracking:**
- Links rewritten through our server
- Track which links clicked
- Timestamp recorded
- Lead to specific content

### 1.3 Email Cleanup

**Auto-Delete Policy:**
- Emails with no reply after 2 years → auto-archive
- User notified before deletion
- Option to keep important emails
- Archived emails retrievable for 30 days

---

## 2. Lead Intelligence

### 2.1 Lead Analysis

For each lead, AI generates:

**Pros (Strengths):**
- Why this lead is valuable
- Positive signals detected
- Alignment with user's offering

**Cons (Challenges):**
- Potential obstacles
- Things to prepare for
- Mitigation strategies

**Opportunities:**
- How they could help user
- Potential deal value
- Collaboration possibilities
- Network value

### 2.2 Skills Matching

**User Skills → Lead Requirements:**
- Extract required skills from job/lead
- Match against user's skills
- Identify gaps
- Suggest which CV to use

---

## 3. User Profile System

### 3.1 Skills Inventory

**Skill Levels:**
- Beginner (0-25%)
- Intermediate (25-50%)
- Advanced (50-75%)
- Expert (75-100%)

**Skill Sources:**
- User self-assessment
- CV parsing
- Interaction analysis
- Endorsements

### 3.2 Learning Recommendations

**AI-Generated Based On:**
- Skills gaps with leads
- Industry trends
- Career goals
- Success patterns

**Resource Sources:**
- YouTube (free)
- edX (free/paid)
- Coursera (free audit)
- Udemy (paid)
- Books
- Articles

### 3.3 Progress Tracking

**Metrics:**
- Leads contacted
- Emails sent
- Open rate
- Reply rate
- Meetings booked
- Deals won
- Revenue generated

**Visualizations:**
- Weekly/monthly trends
- Comparison to goals
- Industry benchmarks

---

## 4. AI Content Generation

### 4.1 Claude API (Quality)

**Use Cases:**
- Professional email writing
- CV improvement
- Cover letters
- Proposals
- Long-form content

**Prompts Include:**
- Lead context
- User profile
- Communication history
- Desired tone
- Specific requirements

### 4.2 OpenAI API (Speed/Logic)

**Use Cases:**
- Lead scoring
- Quick analysis
- Classification
- Summarization
- Reasoning

---

## 5. Voice + Outreach Integration

### 5.1 Reply Notification

**When Reply Detected:**
1. Parse reply content
2. Update lead status
3. Notify user (push/email)
4. Suggest next action
5. Queue for call prep

### 5.2 Call Preparation

**Desktop App Receives:**
- Lead full profile
- Communication history
- AI analysis (pros/cons)
- Talking points
- Questions to ask
- Objection handlers

### 5.3 Post-Call Flow

**After Call Ends:**
1. Generate summary
2. Extract action items
3. Update lead status
4. Draft follow-up email
5. Schedule next touchpoint

---

## 6. Campaign System

### 6.1 Drip Sequences

**Example Sequence:**
```
Day 0:  Initial outreach
Day 3:  Follow-up (if no open)
Day 5:  Follow-up (if opened, no reply)
Day 10: Value-add content
Day 14: Break-up email
```

**Auto-Stop Triggers:**
- Lead replies
- Lead unsubscribes
- Email bounces
- User manually stops

### 6.2 A/B Testing

**Test Variables:**
- Subject lines
- Email content
- Send times
- CTAs

**Metrics:**
- Open rate
- Click rate
- Reply rate
- Conversion rate

---

## 7. Analytics Dashboard

### 7.1 Email Analytics

- Total sent/delivered/bounced
- Open rate (unique/total)
- Click rate
- Reply rate
- Best performing templates
- Best send times
- Worst performing (to improve)

### 7.2 Lead Analytics

- Lead sources breakdown
- Conversion funnel
- Average time to reply
- Average time to close
- Lost reasons analysis

### 7.3 Personal Analytics

- Activity over time
- Goal progress
- Skill improvements
- Learning completions

---

## 8. Document Management

### 8.1 CV System

**Features:**
- Multiple CVs per user
- Default CV setting
- CV per job type
- AI improvement suggestions
- Version history

### 8.2 AI CV Improvement

**Process:**
1. User uploads CV
2. AI extracts structure
3. Compare to job/lead requirements
4. Generate suggestions
5. Auto-apply improvements
6. User reviews and approves

---

## 9. Scraping System

### 9.1 LinkedIn Scraper

**Data Extracted:**
- Name, title, location
- Current company
- Experience history
- Education
- Skills
- Connections count
- Recent activity

### 9.2 Company Scraper

**Data Extracted:**
- Company name, size
- Industry, location
- Tech stack
- Job openings
- Recent news
- Funding info

### 9.3 Job Posting Scraper

**Data Extracted:**
- Job title, company
- Requirements
- Salary range
- Location/remote
- Posted date
- Application URL

---

## 10. Compliance

### 10.1 GDPR

- Consent tracking
- Data export
- Data deletion
- Privacy policy

### 10.2 CAN-SPAM

- Unsubscribe in every email
- Physical address
- Accurate headers
- Honor opt-outs

### 10.3 Email Best Practices

- Warm-up new domains
- Monitor reputation
- Bounce handling
- Spam score checking

---

*Features are built incrementally. Check PLANS.md for current status.*
