# API Documentation Entry Template

**Purpose**: Standard template for documenting APIs and libraries in `API_DOCUMENTATION_INDEX.md`

---

## Standard Entry Format

```markdown
### Provider/Service Name

- **Website**: <https://example.com>
- **Official Docs**: <https://example.com/docs> (if different from website)
- **GitHub**: <https://github.com/example> (if applicable)
- **Provider**: Company/Organization Name
- **Description**: Brief one-sentence description of what this provides
- **Key Features**:
  - Feature 1
  - Feature 2
  - Feature 3
- **API Types**:
  - REST API
  - WebSocket API
  - FIX API
  - Native C++ APIs
- **Data Coverage**:
  - Asset classes supported
  - Geographic coverage
  - Data types available
- **Auth**: Authentication method (apiKey, OAuth, etc.)
- **API Limits**:
  - Free tier: Limits
  - Paid plans: Higher limits
- **Integration**:
  - Language support
  - SDKs available
  - Documentation links
- **Relevance to Box Spread Trading**:
  - Specific use case 1
  - Specific use case 2
  - Specific use case 3
- **Integration Considerations**:
  - Technical requirement 1
  - Technical requirement 2
  - Cost/pricing considerations
- **Comparison with Current Solutions**:
  - vs. TWS API: Key differences
  - vs. Other solutions: Key differences
- **Use Cases**:
  - Use case 1
  - Use case 2
  - Use case 3
- **Contact**: Contact information (email, website)
- **Pricing**:
  - Free tier: Details
  - Paid plans: Details
- **Note**: Additional context, warnings, or recommendations
```

---

## Required Fields

### Minimum Required

- **Website**: Official website URL
- **Description**: What this provides
- **Relevance to Box Spread Trading**: Why it's relevant

### Recommended Fields

- **Key Features**: List of main features
- **API Types**: Types of APIs available
- **Integration Considerations**: Technical requirements
- **Use Cases**: Specific use cases
- **Note**: Additional context

### Optional Fields

- **GitHub**: If open-source
- **Provider**: Company name
- **Auth**: Authentication details
- **API Limits**: Rate limits
- **Pricing**: Cost information
- **Contact**: Contact information

---

## Formatting Guidelines

### URLs

- Always use angle brackets: `<https://example.com>`
- Include protocol: `https://` not just `example.com`

### Lists

- Use bullet points (`-`) for unordered lists
- Use numbered lists for ordered sequences
- Use bold for field names: `- **Field Name**: Value`

### Sections

- Use `###` for provider/service entries
- Use `####` for subsections within providers
- Use `#####` for sub-subsections

### Emphasis

- Use `✅` for supported features
- Use `⚠️` for warnings or limitations
- Use `❌` for not supported

---

## Examples

### Simple Entry (Market Data Provider)

```markdown
### Alpha Vantage

- **URL**: <https://www.alphavantage.co/>
- **Official API Docs**: <https://www.alphavantage.co/documentation/>
- **Description**: Enterprise-grade stock market data API provider
- **Key Features**:
  - Real-time and historical stock market data
  - 60+ technical indicators
  - Market news API with sentiment analysis
- **Auth**: apiKey required (free tier available)
- **API Limits**:
  - Free tier: 5 API calls per minute, 500 calls per day
- **Relevance to Box Spread Trading**:
  - Complements TWS API with additional market data sources
  - Useful for technical analysis with 60+ indicators
- **Note**: Free tier available but limited. Paid plans start at $49.99/month.
```

### Complex Entry (Trading API)

```markdown
### Interactive Brokers TWS API

- **Official Docs**: <https://interactivebrokers.github.io/tws-api/>
- **GitHub**: <https://github.com/InteractiveBrokers/tws-api>
- **Version**: 10.40.01
- **Key Features**:
  - ✅ Full Protocol Buffers support
  - ✅ Order Recovery: Automatic order resubmission
  - ✅ Enhanced error handling
- **Key Classes**:
  - `EClient` / `EClientSocket`: Client connection
  - `EWrapper`: Callback interface (93+ methods)
- **Ports**:
  - `7497`: Paper Trading (TWS)
  - `7496`: Live Trading (TWS)
- **Relevance to Box Spread Trading**:
  - Primary broker API for options trading
  - Comprehensive options support
  - Global market access
- **Note**: Primary trading API for this project.
```

---

## Consistency Checklist

- [ ] All URLs use angle brackets
- [ ] Field names are bold
- [ ] Lists use consistent formatting
- [ ] Relevance section explains box spread connection
- [ ] Note section provides additional context
- [ ] Comparison section included (if applicable)
- [ ] Contact information included (if applicable)
- [ ] Pricing information included (if applicable)

---

## See Also

- **Full Documentation**: `API_DOCUMENTATION_INDEX.md`
- **Summary**: `API_DOCUMENTATION_SUMMARY.md`
- **Consolidation Plan**: `API_DOCUMENTATION_CONSOLIDATION_PLAN.md`
