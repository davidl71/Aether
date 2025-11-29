# NotebookLM Deep Research Workflow

**Purpose**: Step-by-step workflow for conducting deep research using NotebookLM MCP server and Context7.

**Last Updated**: 2025-11-20

---

## Workflow Overview

```
1. Get Documentation (Context7) → 2. Create Notebook (Manual) → 3. Add Sources → 4. Research (NotebookLM MCP) → 5. Synthesize & Document
```

---

## Step 1: Get Documentation with Context7

Use Context7 to get up-to-date API documentation before creating notebooks.

### For Message Queue Research

**NATS Documentation:**
- Library ID: `/nats-io/nats.docs`
- Topics: performance, latency, multi-language support
- Use Context7 to get latest NATS patterns and best practices

**RabbitMQ Documentation:**
- Library ID: `/rabbitmq/rabbitmq-website`
- Topics: performance, latency, AMQP, multi-language clients
- Use Context7 to get latest RabbitMQ patterns

**Usage:**
```
"Get NATS documentation on performance and multi-language support use context7"
"Get RabbitMQ documentation on latency and AMQP use context7"
```

### For CME Research

**CME/Cboe Sources:**
- Use web search for official whitepapers (Context7 may not have CME-specific docs)
- Add URLs directly to NotebookLM

### For ORATS Research

**ORATS API:**
- Check Context7 for ORATS library documentation
- If not available, use ORATS official documentation URLs

---

## Step 2: Create Notebook in NotebookLM

### Option A: Automated Creation (Recommended)

**Use browser control MCP to automate notebook creation:**

1. **Authenticate manually first** (security requirement):
   - Navigate to notebooklm.google.com
   - Sign in with Google account
   - Complete 2FA if required

2. **Use browser control MCP** to automate:
   ```python
   # Navigate to NotebookLM
   browser_navigate("https://notebooklm.google.com")

   # Get page snapshot
   snapshot = browser_snapshot()

   # Click "Create notebook"
   browser_click(element="Create notebook button", ref="[from snapshot]")

   # Enter notebook name
   browser_type(element="Notebook name", ref="[from snapshot]", text="CME Financing Strategies")

   # Create notebook
   browser_click(element="Create", ref="[from snapshot]")
   ```

3. **Add sources automatically**:
   ```python
   # For each source URL
   browser_click(element="Add source", ref="[from snapshot]")
   browser_click(element="URL option", ref="[from snapshot]")
   browser_type(element="URL input", ref="[from snapshot]", text="https://...")
   browser_click(element="Add", ref="[from snapshot]")
   browser_wait_for(time=5)  # Wait for processing
   ```

**See**: [NOTEBOOKLM_AUTOMATION_GUIDE.md](NOTEBOOKLM_AUTOMATION_GUIDE.md) for complete automation workflow

### Option B: Manual Creation

**If automation is not available:**

1. Go to [notebooklm.google.com](https://notebooklm.google.com)
2. Click "Create notebook"
3. Name the notebook (e.g., "CME Financing Strategies")
4. Add sources manually

### For Each High-Priority Research:

**Notebook 1: CME Financing Strategies**
- Name: "CME Financing Strategies"
- Purpose: Synthesize CME/Cboe whitepapers

**Notebook 2: Message Queue Solutions**
- Name: "Message Queue Solutions"
- Purpose: Compare NATS, RabbitMQ, Redis, ZeroMQ

**Notebook 3: ORATS Options Data**
- Name: "ORATS Options Data"
- Purpose: ORATS API integration patterns

**Notebook 4-6: TWS API Learnings** (Split into 3 notebooks per optimization guide)
- Notebook 4: "TWS API Core Patterns"
- Notebook 5: "TWS API Advanced Topics"
- Notebook 6: "TWS API Integration"

---

## Step 3: Add Sources to Notebook

### Option A: Manual Addition (Web Interface)

1. In NotebookLM web interface, click "Add source"
2. Paste URLs or upload files
3. Wait for processing

### Option B: Via NotebookLM MCP (If Supported)

Check if NotebookLM MCP supports adding sources programmatically. If yes:
- Use MCP tools to add URLs
- Add local markdown files

### Sources by Notebook

**CME Financing Strategies:**
1. https://www.cmegroup.com/articles/whitepapers/capital-efficiencies-and-air-trfs.html
2. https://www.cboe.com/insights/posts/why-consider-box-spreads-as-an-alternative-borrowing-lending-strategy/
3. https://www.cmegroup.com/articles/2025/quantifying-and-hedging-equity-financing-risk.html
4. https://www.cmegroup.com/market-data/license-data/licensed-market-data-distributors.html

**Message Queue Solutions:**
1. NATS Documentation (from Context7 or https://docs.nats.io/)
2. RabbitMQ Documentation (from Context7 or https://www.rabbitmq.com/docs/)
3. Redis Streams: https://redis.io/docs/data-types/streams/
4. ZeroMQ: https://zeromq.org/

**ORATS Options Data:**
1. ORATS API Documentation: https://orats.com/docs
2. ORATS Data API: https://orats.com/data-api

---

## Step 4: Share Notebook and Add to Library

### Share Notebook:

1. In NotebookLM, click **⚙️ Share**
2. Select **"Anyone with link"**
3. Click **Copy link**
4. Save the notebook link

### Add to Library (Via MCP or Manual):

**Via Cursor Chat:**
```
"Add [notebook-link] to library tagged '[topic], research, [category]'"
```

Example:
```
"Add https://notebooklm.google.com/notebook/abc123 to library tagged 'cme, financing, research, external'"
```

---

## Step 5: Research Using NotebookLM MCP

Once notebooks are created and added to library, use NotebookLM MCP to query them.

### Query Patterns:

**For CME Financing:**
```
"Research CME financing strategies in NotebookLM and compare box spreads vs AIR TRFs vs futures financing for capital efficiency"
```

**For Message Queues:**
```
"Research message queue solutions in NotebookLM and compare NATS, RabbitMQ, Redis Streams, and ZeroMQ for sub-millisecond trading systems"
```

**For ORATS:**
```
"Research ORATS options data in NotebookLM and summarize key integration patterns for box spread detection"
```

### Synthesis Queries:

1. **Comparison Queries:**
   - "Compare [option A] vs [option B] for [use case]"
   - "What are the key differences between [solutions]?"

2. **Analysis Queries:**
   - "What are the capital efficiency implications of each financing strategy?"
   - "Which message queue solution has the best multi-language support?"

3. **Integration Queries:**
   - "Summarize the integration requirements for [service]"
   - "What are the top 3 implementation considerations?"

---

## Step 6: Document Findings

### Update Research Documents:

After NotebookLM research, update the original research documents with synthesized findings.

**Format:**
```markdown
## Synthesis (NotebookLM Research)

**Date**: 2025-11-20
**Notebook**: [Notebook Name/Link]

### Key Findings:

1. [Finding 1 with citation]
2. [Finding 2 with citation]
3. [Finding 3 with citation]

### Recommendations:

- [Recommendation based on synthesis]
- [Implementation considerations]

### Sources:
- [Source 1]
- [Source 2]
```

### Create Consolidated Documents:

For TWS API research, create:
- `docs/research/learnings/TWS_API_BEST_PRACTICES_CONSOLIDATED.md`

---

## Context7 + NotebookLM Integration Strategy

### When to Use Context7:

- **API Documentation**: Get latest library/framework docs
- **Code Examples**: Get up-to-date code patterns
- **Best Practices**: Get current best practices (2025)

### When to Use NotebookLM:

- **External Whitepapers**: CME/Cboe research papers
- **Multiple Sources**: Synthesize information from multiple documents
- **Research Papers**: Academic or industry research
- **Video Content**: YouTube tutorials and talks

### Combined Workflow:

1. **Get API Docs** (Context7) → Understand current patterns
2. **Create Notebook** (NotebookLM) → Add external sources
3. **Research** (NotebookLM MCP) → Synthesize findings
4. **Document** → Update research documents with insights

---

## Example: Complete CME Research Workflow

### Step 1: Get Context7 Documentation (If Available)

```
"Get CME Group API documentation use context7"
```

### Step 2: Create Notebook

1. Go to NotebookLM
2. Create "CME Financing Strategies"
3. Share and copy link

### Step 3: Add Sources

Add 4 external URLs:
- CME Capital Efficiencies whitepaper
- Cboe Box Spreads article
- CME Equity Financing Risk whitepaper
- CME Market Data Distributors

### Step 4: Add to Library

```
"Add [notebook-link] to library tagged 'cme, financing, research, external'"
```

### Step 5: Research

```
"Research CME financing strategies in NotebookLM and answer:
1. Compare box spreads vs AIR TRFs vs futures financing for capital efficiency
2. What are the counterparty risk differences?
3. Summarize integration requirements for CME market data feeds"
```

### Step 6: Document

Update `docs/research/external/CME_RESEARCH.md` with synthesis section.

---

## Troubleshooting

### NotebookLM MCP Not Working:

1. Check authentication: "Log me in to NotebookLM"
2. Verify notebook is shared: "Anyone with link"
3. Check notebook link is correct
4. Verify notebook is in library

### Context7 Not Finding Library:

1. Try different library name variations
2. Check if library exists in Context7
3. Use web search as fallback
4. Use official documentation URLs in NotebookLM

### Sources Not Processing:

1. Check URL accessibility
2. Verify authentication for protected sources
3. Wait for processing (may take several minutes)
4. Try re-adding source

---

## See Also

- **[NOTEBOOKLM_RESEARCH_SETUP.md](NOTEBOOKLM_RESEARCH_SETUP.md)** - Initial setup guide
- **[NOTEBOOKLM_OPTIMIZATION_GUIDE.md](NOTEBOOKLM_OPTIMIZATION_GUIDE.md)** - Optimization strategies
- **[RESEARCH_INDEX.md](../../research/../RESEARCH_INDEX.md)** - Master research index

---

**Last Updated**: 2025-11-20
**Maintained by**: AI Assistant
