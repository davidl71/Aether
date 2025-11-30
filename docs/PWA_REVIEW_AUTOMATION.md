# PWA Review Automation Guide

**Date**: 2025-11-20
**Purpose**: Guide for setting up and using automated PWA review analysis

---

## Overview

The automated PWA review system analyzes the current PWA state, compares it against investment strategy goals, and generates an improvement analysis document. This can run on a schedule (cron or GitHub Actions) to keep the analysis up-to-date.

---

## Quick Start

### 1. Manual Run

Run the analysis script manually:

```bash

# Basic run (no AI insights)

python3 scripts/automate_pwa_review.py

# With custom config

python3 scripts/automate_pwa_review.py --config scripts/pwa_review_config.json

# With custom output path

python3 scripts/automate_pwa_review.py --output docs/my_analysis.md
```

### 2. Enable AI Insights (Optional)

To get AI-generated insights, configure an API provider:

**Option A: OpenAI**

```bash
export OPENAI_API_KEY="your-api-key-here"
```

Edit `scripts/pwa_review_config.json`:

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

Edit `scripts/pwa_review_config.json`:

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

## Scheduling Options

### Option 1: Local Cron Job (Recommended for Development)

Set up a cron job to run the analysis automatically:

```bash

# Run weekly on Sunday at 2:00 AM

./scripts/setup_pwa_review_cron.sh weekly sunday 02:00

# Run daily at 4:00 AM

./scripts/setup_pwa_review_cron.sh daily 04:00

# Run monthly on the 1st at 3:00 AM

./scripts/setup_pwa_review_cron.sh monthly 1 03:00
```

**View cron jobs:**

```bash
crontab -l
```

**Remove cron job:**

```bash
crontab -l | grep -v 'run_pwa_review_cron.sh' | crontab -
```

**Logs:**

- Success logs: `scripts/pwa_review.log`
- Error logs: `scripts/pwa_review_errors.log`

### Option 2: GitHub Actions (Recommended for Production)

The GitHub Actions workflow runs automatically on a schedule and commits updates.

**Enable the workflow:**

1. The workflow file is already created: `.github/workflows/pwa-review-scheduled.yml`
2. It runs weekly on Sunday at 02:00 UTC
3. Results are automatically committed to the repository

**Manual trigger:**

- Go to Actions tab in GitHub
- Select "Scheduled PWA Review Analysis"
- Click "Run workflow"

**Configure schedule:**
Edit `.github/workflows/pwa-review-scheduled.yml`:

```yaml
schedule:
  - cron: '0 2 * * 0'  # Change to your preferred schedule
```

**Add AI API keys (optional):**

1. Go to repository Settings → Secrets and variables → Actions
2. Add secrets:
   - `OPENAI_API_KEY` (if using OpenAI)
   - `ANTHROPIC_API_KEY` (if using Anthropic)
3. Uncomment the env section in the workflow file

---

## Configuration

### Configuration File: `scripts/pwa_review_config.json`

```json
{
  "ai_api": {
    "provider": "none",
    "model": "gpt-4",
    "api_key": null
  },
  "output_path": "docs/PWA_IMPROVEMENT_ANALYSIS.md"
}
```

**Options:**

- `ai_api.provider`: `"none"`, `"openai"`, or `"anthropic"`
- `ai_api.model`: Model name (e.g., `"gpt-4"`, `"claude-3-5-sonnet-20241022"`)
- `ai_api.api_key`: Leave `null` and use environment variables for security
- `output_path`: Path to write the analysis document

---

## What the Script Does

1. **Loads Todo2 Tasks**: Reads `.todo2/state.todo2.json` to analyze task alignment
2. **Analyzes PWA Structure**: Scans `web/` directory for components, hooks, API integrations
3. **Checks Goal Alignment**: Compares current state against primary goals:
   - Unified position view
   - Cash flow modeling
   - Opportunity simulation
   - Relationship visualization
4. **Generates Insights**: Uses AI API (if configured) or basic analysis
5. **Writes Analysis**: Updates `docs/PWA_IMPROVEMENT_ANALYSIS.md`

---

## Output

The script generates/updates `docs/PWA_IMPROVEMENT_ANALYSIS.md` with:

- Current PWA state analysis
- Todo2 task alignment statistics
- AI-generated insights (if enabled)
- Recommendations and next steps

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
chmod +x scripts/automate_pwa_review.py
chmod +x scripts/setup_pwa_review_cron.sh
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
./scripts/run_pwa_review_cron.sh
```

**Verify cron entry:**

```bash
crontab -l | grep pwa_review
```

### GitHub Actions Not Running

**Check workflow status:**

- Go to Actions tab in GitHub
- Look for "Scheduled PWA Review Analysis" workflow
- Check for errors in workflow logs

**Verify schedule:**

- GitHub Actions uses UTC time
- Check that the cron expression is valid
- Workflows may be delayed during high load

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
2. **Review output**: Check the generated analysis before enabling auto-commit
3. **Set appropriate schedule**: Weekly is usually sufficient
4. **Monitor logs**: Check logs regularly for errors
5. **Secure API keys**: Always use environment variables, never commit keys
6. **Version control**: Commit the analysis document to track changes over time

---

## Advanced Usage

### Custom Analysis

Modify `scripts/automate_pwa_review.py` to:

- Add custom analysis logic
- Check additional files/directories
- Generate different output formats
- Integrate with other tools

### Integration with Other Tools

**Combine with notification systems:**

- Add email notifications on completion
- Send Slack messages with summary
- Create GitHub issues for high-priority items

**Combine with CI/CD:**

- Run analysis before deployments
- Block deployments if alignment drops below threshold
- Generate reports for stakeholders

---

## References

- `scripts/automate_pwa_review.py` - Main analysis script
- `scripts/pwa_review_config.json` - Configuration file
- `scripts/setup_pwa_review_cron.sh` - Cron setup script
- `.github/workflows/pwa-review-scheduled.yml` - GitHub Actions workflow
- `docs/PWA_IMPROVEMENT_ANALYSIS.md` - Generated analysis document

---

*This automation helps keep PWA analysis up-to-date and aligned with investment strategy goals.*
