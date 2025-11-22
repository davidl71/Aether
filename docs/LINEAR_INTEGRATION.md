# Linear Integration Guide

This document describes how to integrate with Linear.app for issue tracking and project management.

## Overview

Two integration methods are available:

1. **GitKraken MCP** (via Cursor) - Issue tracking through MCP server
2. **Direct Linear API Client** (Python) - Full programmatic access to Linear's GraphQL API

## Method 1: GitKraken MCP Integration

### Setup

GitKraken MCP is already configured in `.cursor/mcp.json`. This provides issue tracking capabilities through Cursor's AI assistant.

### Usage

Once Cursor is restarted, you can use Linear through GitKraken MCP:

- Create Linear issues from code changes
- Link commits to Linear issues
- Track issues related to features
- Manage PRs with Linear issue references

### Configuration

No additional configuration needed - GitKraken MCP handles Linear authentication through your GitKraken account.

## Method 2: Direct Linear API Client

### Setup

1. **Get a Linear API Key**:
   - Go to Linear Settings → API
   - Create a Personal Access Token
   - Copy the token

2. **Set Environment Variable**:

   ```bash
   export LINEAR_API_KEY="your_personal_access_token_here"
   ```

   Or add to your `.env` file:

   ```
   LINEAR_API_KEY=your_personal_access_token_here
   ```

### Basic Usage

```python
from python.integration.linear_client import LinearClient

# Initialize client (reads LINEAR_API_KEY from environment)
client = LinearClient()

# Get all teams
teams = client.get_teams()
print(f"Found {len(teams)} teams")

# Get issues for a team
team_id = teams[0]["id"]
issues = client.get_issues(team_id=team_id, state="In Progress")
print(f"Found {len(issues)} in-progress issues")

# Create a new issue
new_issue = client.create_issue(
    team_id=team_id,
    title="Fix box spread calculation bug",
    description="The APR calculation is incorrect for wide spreads",
    priority=1,  # High priority
)
print(f"Created issue: {new_issue['identifier']}")

# Update an issue
client.update_issue(
    issue_id=new_issue["id"],
    state_id="done_state_id",  # Move to Done
)

# Add a comment
client.add_comment(
    issue_id=new_issue["id"],
    body="Fixed in commit abc123",
)
```

### API Methods

#### Teams

- `get_teams()` - List all teams in workspace

#### Issues

- `get_issues(team_id, assignee_id, state, limit)` - Query issues with filters
- `get_issue(issue_id)` - Get specific issue by ID or identifier (e.g., "ENG-123")
- `create_issue(...)` - Create new issue
- `update_issue(...)` - Update existing issue
- `add_comment(issue_id, body)` - Add comment to issue

#### Workflow States

- `get_states(team_id)` - Get workflow states (Backlog, In Progress, Done, etc.)

### Integration with Trading System

You can integrate Linear issue tracking with your trading system:

```python
from python.integration.linear_client import LinearClient

def log_trading_error_to_linear(error: str, context: dict):
    """Log trading errors to Linear for tracking."""
    client = LinearClient()

    # Find or create "Trading Errors" team
    teams = client.get_teams()
    trading_team = next((t for t in teams if "Trading" in t["name"]), None)

    if not trading_team:
        return

    # Create issue
    issue = client.create_issue(
        team_id=trading_team["id"],
        title=f"Trading Error: {error[:50]}",
        description=f"""
        Error: {error}

        Context:
        - Timestamp: {context.get('timestamp')}
        - Strategy: {context.get('strategy')}
        - Symbol: {context.get('symbol')}
        """,
        priority=0,  # Urgent
    )

    return issue["identifier"]
```

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `LINEAR_API_KEY` | Personal access token from Linear | Yes |

## GraphQL API Reference

Linear uses GraphQL for all API operations. The client abstracts common operations, but you can extend it for custom queries:

```python
# Custom GraphQL query
custom_query = """
query {
  issues(filter: { priority: { eq: 0 } }) {
    nodes {
      id
      title
      priority
    }
  }
}
"""
data = client._query(custom_query)
```

See [Linear's GraphQL API Documentation](https://developers.linear.app/docs/graphql/working-with-the-graphql-api) for full schema.

## Best Practices

1. **Error Handling**: Always wrap Linear API calls in try/except blocks
2. **Rate Limiting**: Linear has rate limits - implement retry logic for production
3. **Caching**: Cache team/state IDs to reduce API calls
4. **Security**: Never commit Linear API keys to version control
5. **Testing**: Use Linear's test workspace for development

## Troubleshooting

### "Missing Linear API key" error

- Ensure `LINEAR_API_KEY` environment variable is set
- Verify the token is valid in Linear Settings → API

### "Linear API errors" exception

- Check the error message for specific GraphQL errors
- Verify team IDs, state IDs, and other references are correct
- Check Linear's status page for API outages

### Authentication failures

- Regenerate your personal access token
- Ensure token has necessary permissions (read/write issues)

## See Also

- [Linear API Documentation](https://developers.linear.app/docs)
- [GraphQL API Reference](https://developers.linear.app/docs/graphql/working-with-the-graphql-api)
- [MCP Extensions Integration Guide](MCP_EXTENSIONS_INTEGRATION.md)
