# NotebookLM Research Setup Guide

**Purpose**: Step-by-step guide for setting up NotebookLM notebooks to synthesize high-priority research documents with external sources.

**Last Updated**: 2025-11-20

---

## High-Priority Notebooks

Based on `docs/RESEARCH_INDEX.md`, the following notebooks should be created for deep research synthesis:

### 1. CME Financing Strategies ⭐⭐⭐

**Source Document**: `docs/research/external/CME_RESEARCH.md`

**External URLs to Add**:
1. CME Group – Capital Efficiencies and AIR TRFs
   - https://www.cmegroup.com/articles/whitepapers/capital-efficiencies-and-air-trfs.html
2. Cboe – Box Spreads as Alternative Borrowing & Lending
   - https://www.cboe.com/insights/posts/why-consider-box-spreads-as-an-alternative-borrowing-lending-strategy/
3. CME Group – Quantifying and Hedging Equity Financing Risk
   - https://www.cmegroup.com/articles/2025/quantifying-and-hedging-equity-financing-risk.html
4. CME Licensed Market Data Distributors
   - https://www.cmegroup.com/market-data/license-data/licensed-market-data-distributors.html

**Synthesis Queries**:
- "Compare box spreads vs AIR TRFs vs futures financing for capital efficiency"
- "What are the key differences in counterparty risk between box spreads and other financing methods?"
- "Summarize the integration requirements for CME market data feeds"
- "What are the capital efficiency implications of each financing strategy?"

**Expected Output**: Update `CME_RESEARCH.md` with synthesized findings section

---

### 2. Message Queue Solutions ⭐⭐⭐

**Source Document**: `docs/research/architecture/MESSAGE_QUEUE_RESEARCH.md`

**External URLs to Add**:
1. NATS Documentation
   - https://docs.nats.io/
2. RabbitMQ Documentation
   - https://www.rabbitmq.com/docs/
3. Redis Streams Documentation
   - https://redis.io/docs/data-types/streams/
4. ZeroMQ Documentation
   - https://zeromq.org/

**Synthesis Queries**:
- "Compare NATS, RabbitMQ, Redis Streams, and ZeroMQ for trading system message queues"
- "What are the latency characteristics of each solution?"
- "Which solution is best for multi-language coordination (C++, Python, Rust, Go, TypeScript)?"
- "Summarize the deployment complexity for each solution"

**Expected Output**: Update `MESSAGE_QUEUE_RESEARCH.md` with synthesis section

---

### 3. ORATS Options Data ⭐⭐⭐

**Source Document**: `docs/research/external/ORATS_INTEGRATION.md`

**External URLs to Add**:
1. ORATS API Documentation
   - https://orats.com/docs
2. ORATS Data API
   - https://orats.com/data-api
3. ORATS Integration Guides
   - (Add any additional ORATS documentation URLs)

**Synthesis Queries**:
- "What are the key features of ORATS options data APIs?"
- "How can ORATS enhance box spread detection and execution?"
- "What are the integration requirements for ORATS APIs?"
- "Compare ORATS with other options data providers"

**Expected Output**: Update `ORATS_INTEGRATION.md` with synthesis section

---

### 4. TWS API Best Practices ⭐⭐⭐

**Source Documents**: Multiple learnings documents in `docs/research/learnings/`

**Documents to Add**:
1. `TWS_API_BEST_PRACTICES.md`
2. `TWS_API_CODE_EXAMPLES_LEARNINGS.md`
3. `TWS_API_MARKET_DATA_LEARNINGS.md`
4. `TWS_API_TROUBLESHOOTING_LEARNINGS.md`
5. `TWS_API_DOCKER_LEARNINGS.md`
6. `TWS_API_IMPLEMENTATION_COMPARISON.md`
7. `IB_API_QUICK_REFERENCE_LEARNINGS.md`
8. `IB_ASYNC_LEARNINGS.md`
9. `IBC_LEARNINGS.md`
10. `ECLIENT_EWRAPPER_ARCHITECTURE.md`

**Synthesis Queries**:
- "Consolidate all TWS API best practices into a unified guide"
- "What are the common patterns for EClient/EWrapper implementation?"
- "What are the best practices for handling market data in TWS API?"
- "Summarize troubleshooting strategies for common TWS API issues"
- "What are the recommended patterns for async TWS API operations?"

**Expected Output**: Create `docs/research/learnings/TWS_API_BEST_PRACTICES_CONSOLIDATED.md`

---

### 5. Trading Framework Evaluation ⭐⭐

**Source Document**: `docs/research/analysis/TRADING_FRAMEWORK_EVALUATION.md`

**Synthesis Queries**:
- "Compare trading frameworks for box spread trading"
- "What are the key criteria for evaluating trading frameworks?"
- "Which frameworks are best suited for multi-leg options strategies?"

**Expected Output**: Update `TRADING_FRAMEWORK_EVALUATION.md` with synthesis section

---

## Setup Instructions

### Step 1: Create Notebook in NotebookLM

1. Go to [notebooklm.google.com](https://notebooklm.google.com)
2. Click "Create notebook"
3. Name the notebook (e.g., "CME Financing Strategies")
4. Add sources:
   - **For URLs**: Paste URLs directly
   - **For local documents**: Upload markdown files or copy content
5. Wait for processing (may take a few minutes)

### Step 2: Share Notebook

1. Click **⚙️ Share** button
2. Select **"Anyone with link"**
3. Click **Copy link**
4. Save the link for Step 3

### Step 3: Add Notebook to Library

In Cursor chat, say:

```
"Add [notebook-link] to library tagged '[topic], research, [category]'"
```

Example:
```
"Add https://notebooklm.google.com/notebook/abc123 to library tagged 'cme, financing, research, external'"
```

### Step 4: Research and Synthesize

In Cursor chat, say:

```
"Research [topic] in NotebookLM and update [document-path] with synthesized findings"
```

Example:
```
"Research CME financing strategies in NotebookLM and update docs/research/external/CME_RESEARCH.md with synthesized findings"
```

The AI will:
1. Query NotebookLM with synthesis questions
2. Get citation-backed answers
3. Update the research document with synthesized insights
4. Add a "Synthesis" section with key findings

---

## Workflow Summary

```
1. Create Notebook (NotebookLM web) → 2. Share Notebook → 3. Add to Library (Cursor) → 4. Research & Synthesize (Cursor)
```

---

## Benefits

- **Zero Hallucination**: Answers grounded in actual sources
- **Citation-Backed**: Direct references to source material
- **Synthesis**: Combines information from multiple sources
- **Efficiency**: Faster than manual research and synthesis

---

## Optimization for NotebookLM Limitations

**⚠️ Important**: NotebookLM has limits that require strategic planning:

- **Free Plan**: 50 sources per notebook, ~25 million words total
- **Per Source**: 500,000 words or 200 MB maximum
- **NotebookLM Plus**: 300 sources, 150 million words (paid)

### Quick Optimization Tips

1. **Split Large Research**: TWS API learnings (10+ documents) should be split into 2-3 notebooks
2. **Prioritize External Sources**: Focus on documents with external links first
3. **Combine Related Documents**: Merge smaller related docs to save source slots
4. **Create Summary Notebooks**: Use one notebook to synthesize findings from multiple notebooks

**See**: [NOTEBOOKLM_OPTIMIZATION_GUIDE.md](NOTEBOOKLM_OPTIMIZATION_GUIDE.md) for detailed strategies

---

## See Also

- **[NOTEBOOKLM_OPTIMIZATION_GUIDE.md](NOTEBOOKLM_OPTIMIZATION_GUIDE.md)** - Detailed optimization strategies
- **[NOTEBOOKLM_USAGE.md](../NOTEBOOKLM_USAGE.md)** - Complete NotebookLM usage guide
- **[RESEARCH_INDEX.md](../RESEARCH_INDEX.md)** - Master research index
- **[MCP_SERVERS.md](../MCP_SERVERS.md)** - MCP server configuration
