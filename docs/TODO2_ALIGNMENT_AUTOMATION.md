# Todo2 Alignment Automation Guide

**Date**: 2025-11-20
**Purpose**: Guide for setting up and using automated Todo2 task alignment analysis

---

## Overview

The automated Todo2 alignment system analyzes task priorities against the investment strategy framework, identifies misaligned tasks, and generates an alignment analysis document. This runs on a schedule (cron) to keep task priorities aligned with strategic goals.

---

## Quick Start

### 1. Manual Run

Run the analysis script manually:

```bash
# Basic run (no AI insights)
python3 scripts/automate_todo2_alignment.py

# With custom config
python3 scripts/automate_todo2_alignment.py --config scripts/todo2_alignment_config.json

# With custom output path
python3 scripts/automate_todo2_alignment.py --output docs/my_alignment.md
```

### 2. Enable AI Insights (Optional)

To get AI-generated insights, configure an API provider:

**Option A: OpenAI**
```bash
export OPENAI_API_KEY="your-api-key-here"
```

Edit `scripts/todo2_alignment_config.json`:
```json
{
  "ai_api": {
    "provider": "openai",
    "model": "gpt-4"
  }
}
```

Install OpenAI library:
```bash
pip install openai
```

**Option B: Anthropic**
```bash
export ANTHROPIC_API_KEY="your-api-key-here"
```

Edit `scripts/todo2_alignment_config.json`:
```json
{
  "ai_api": {
    "provider": "anthropic",
    "model": "claude-3-5-sonnet-20241022"
  }
}
```

Install Anthropic library:
```bash
pip install anthropic
```

---

## Scheduling

### Local Cron Job

Set up a cron job to run the analysis automatically:

```bash
# Run weekly on Monday at 2:00 AM
./scripts/setup_todo2_alignment_cron.sh weekly monday 02:00

# Run daily at 4:00 AM
./scripts/setup_todo2_alignment_cron.sh daily 04:00

# Run monthly on the 1st at 3:00 AM
./scripts/setup_todo2_alignment_cron.sh monthly 1 03:00
```

**View cron jobs:**
```bash
crontab -l
```

**Remove cron job:**
```bash
crontab -l | grep -v 'run_todo2_alignment_cron.sh' | crontab -
```

**Logs:**
- Success logs: `scripts/todo2_alignment.log`
- Error logs: `scripts/todo2_alignment_errors.log`

---

## Configuration

### Configuration File: `scripts/todo2_alignment_config.json`

```json
{
  "ai_api": {
    "provider": "none",
    "model": "gpt-4",
    "api_key": null
  },
  "output_path": "docs/TODO2_PRIORITY_ALIGNMENT_ANALYSIS.md"
}
```

**Options:**
- `ai_api.provider`: `"none"`, `"openai"`, or `"anthropic"`
- `ai_api.model`: Model name (e.g., `"gpt-4"`, `"claude-3-5-sonnet-20241022"`)
- `ai_api.api_key`: Leave `null` and use environment variables for security
- `output_path`: Path to write the analysis document

---

## What the Script Does

1. **Loads Todo2 Tasks**: Reads `.todo2/state.todo2.json` to analyze all tasks
2. **Analyzes Strategy Alignment**: Compares tasks against investment strategy framework phases:
   - Phase 1: Foundation (portfolio aggregation, position sources)
   - Phase 2: Core Calculations (cash flow, Greeks, convexity)
   - Phase 3: Advanced Features (cash management, T-bill ladder, ETF)
3. **Identifies Issues**:
   - Misaligned tasks (high priority but not strategy-related)
   - Stale tasks (no updates in 30+ days)
   - Blocked tasks (dependencies not complete)
4. **Calculates Alignment Score**: 0-100% based on multiple factors
5. **Generates Insights**: Uses AI API (if configured) or basic analysis
6. **Writes Analysis**: Updates `docs/TODO2_PRIORITY_ALIGNMENT_ANALYSIS.md`

---

## Output

The script generates/updates `docs/TODO2_PRIORITY_ALIGNMENT_ANALYSIS.md` with:

- Executive summary with alignment score
- Task distribution by priority and status
- Phase-specific task breakdown
- Strategy-critical tasks list
- Misaligned, stale, and blocked tasks
- AI-generated insights (if enabled)
- Recommendations for improvement

### Alignment Score Calculation

The alignment score (0-100%) considers:
- **40%**: Strategy-critical tasks are high priority
- **30%**: High priority tasks are strategy-aligned
- **20%**: Tasks are not stale
- **10%**: Tasks are not blocked

**Target: 80%+ alignment**

---

## Troubleshooting

### Script Fails to Run

**Check Python version:**
```bash
python3 --version  # Should be 3.8+
```

**Check dependencies:**
```bash
# If using AI features
pip install openai  # or anthropic
```

**Check file permissions:**
```bash
chmod +x scripts/automate_todo2_alignment.py
chmod +x scripts/setup_todo2_alignment_cron.sh
```

### Cron Job Not Running

**Check cron logs:**
```bash
# macOS
grep CRON /var/log/system.log

# Linux
grep CRON /var/log/syslog
```

**Test cron script manually:**
```bash
./scripts/run_todo2_alignment_cron.sh
```

**Verify cron entry:**
```bash
crontab -l | grep todo2_alignment
```

### Low Alignment Score

**Common causes:**
- High-priority tasks not aligned with strategy
- Many stale tasks
- Many blocked tasks

**Solutions:**
- Review misaligned tasks and adjust priorities
- Update or complete stale tasks
- Complete dependencies to unblock tasks
- Focus on strategy-critical tasks

### AI API Errors

**Check API key:**
```bash
echo $OPENAI_API_KEY  # or $ANTHROPIC_API_KEY
```

**Test API connection:**
```python
import openai  # or anthropic
client = openai.OpenAI(api_key=os.getenv('OPENAI_API_KEY'))
# Test call...
```

**Fallback to basic insights:**
- Set `"provider": "none"` in config
- Script will use basic analysis without AI

---

## Best Practices

1. **Start without AI**: Test the script with `"provider": "none"` first
2. **Review output**: Check the generated analysis before making changes
3. **Set appropriate schedule**: Weekly is usually sufficient
4. **Monitor logs**: Check logs regularly for errors
5. **Secure API keys**: Always use environment variables, never commit keys
6. **Version control**: Commit the analysis document to track changes over time
7. **Act on recommendations**: Use insights to improve task alignment

---

## Integration with Other Tools

### Combine with PWA Review

Run both analyses together:

```bash
# Create combined script
cat > scripts/run_all_analyses.sh << 'EOF'
#!/bin/bash
python3 scripts/automate_pwa_review.py
python3 scripts/automate_todo2_alignment.py
EOF
chmod +x scripts/run_all_analyses.sh
```

### Pre-commit Hook

Add to pre-commit to check alignment before commits:

```bash
# In .git/hooks/pre-commit
python3 scripts/automate_todo2_alignment.py --output /tmp/alignment_check.md
# Check alignment score and warn if below threshold
```

---

## Advanced Usage

### Custom Analysis

Modify `scripts/automate_todo2_alignment.py` to:
- Add custom alignment criteria
- Check additional task attributes
- Generate different output formats
- Integrate with other tools

### Integration with Notification Systems

**Combine with notification systems:**
- Add email notifications on low alignment scores
- Send Slack messages with summary
- Create GitHub issues for misaligned tasks

**Example:**
```python
# In automate_todo2_alignment.py
if alignment_score < 70:
    send_notification(f"Todo2 alignment dropped to {alignment_score}%")
```

---

## References

- `scripts/automate_todo2_alignment.py` - Main analysis script
- `scripts/todo2_alignment_config.json` - Configuration file
- `scripts/setup_todo2_alignment_cron.sh` - Cron setup script
- `docs/TODO2_PRIORITY_ALIGNMENT_ANALYSIS.md` - Generated analysis document
- `docs/INVESTMENT_STRATEGY_FRAMEWORK.md` - Strategy framework reference
- `docs/TODO2_PRIORITIZED_ACTION_PLAN.md` - Action plan reference

---

*This automation helps keep Todo2 task priorities aligned with investment strategy goals automatically.*
