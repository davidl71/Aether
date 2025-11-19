"""
linear_client.py - Linear.app GraphQL API client for issue tracking

Reads API credentials from environment variables:
- LINEAR_API_KEY: Personal access token from Linear settings

Linear API Documentation: https://developers.linear.app/docs/graphql/working-with-the-graphql-api
"""

from __future__ import annotations

import os
from datetime import datetime, timezone
from typing import Dict, List, Optional, Any

import requests


class LinearClient:
  """
  Client for Linear.app GraphQL API.

  Linear uses GraphQL for all API operations. This client provides
  a simplified interface for common issue tracking operations.
  """

  GRAPHQL_ENDPOINT = "https://api.linear.app/graphql"

  def __init__(
    self,
    api_key: Optional[str] = None,
    session: Optional[requests.Session] = None,
  ) -> None:
    """
    Initialize Linear client.

    Args:
      api_key: Linear personal access token (or from LINEAR_API_KEY env var)
      session: Optional requests.Session for connection pooling
    """
    self.api_key = api_key or os.getenv("LINEAR_API_KEY", "")
    if not self.api_key:
      raise RuntimeError(
        "Missing Linear API key (LINEAR_API_KEY environment variable)"
      )

    self._session = session or requests.Session()
    self._session.headers.update(
      {
        "Authorization": self.api_key,
        "Content-Type": "application/json",
      }
    )

  def _query(self, query: str, variables: Optional[Dict] = None) -> Dict:
    """
    Execute a GraphQL query/mutation.

    Args:
      query: GraphQL query string
      variables: Optional variables for the query

    Returns:
      JSON response from Linear API
    """
    payload = {"query": query}
    if variables:
      payload["variables"] = variables

    resp = self._session.post(
      self.GRAPHQL_ENDPOINT,
      json=payload,
      timeout=10,
    )
    resp.raise_for_status()
    data = resp.json()

    if "errors" in data:
      error_messages = [e.get("message", "Unknown error") for e in data["errors"]]
      raise RuntimeError(f"Linear API errors: {', '.join(error_messages)}")

    return data

  def get_teams(self) -> List[Dict[str, Any]]:
    """
    Get all teams in the workspace.

    Returns:
      List of team dictionaries with id, name, key, etc.
    """
    query = """
    query {
      teams {
        nodes {
          id
          name
          key
          description
        }
      }
    }
    """
    data = self._query(query)
    return data.get("data", {}).get("teams", {}).get("nodes", [])

  def get_issues(
    self,
    team_id: Optional[str] = None,
    assignee_id: Optional[str] = None,
    state: Optional[str] = None,
    limit: int = 50,
  ) -> List[Dict[str, Any]]:
    """
    Get issues matching filters.

    Args:
      team_id: Filter by team ID
      assignee_id: Filter by assignee user ID
      state: Filter by state (e.g., "In Progress", "Done")
      limit: Maximum number of issues to return

    Returns:
      List of issue dictionaries
    """
    filters = []
    if team_id:
      filters.append(f'team: {{ id: {{ eq: "{team_id}" }} }}')
    if assignee_id:
      filters.append(f'assignee: {{ id: {{ eq: "{assignee_id}" }} }}')
    if state:
      filters.append(f'state: {{ name: {{ eq: "{state}" }} }}')

    filter_str = ", ".join(filters) if filters else ""

    query = f"""
    query {{
      issues(
        filter: {{ {filter_str} }},
        first: {limit}
      ) {{
        nodes {{
          id
          identifier
          title
          description
          priority
          state {{
            id
            name
            type
          }}
          assignee {{
            id
            name
            email
          }}
          team {{
            id
            name
            key
          }}
          createdAt
          updatedAt
          dueDate
          labels {{
            nodes {{
              id
              name
              color
            }}
          }}
        }}
      }}
    }}
    """
    data = self._query(query)
    return data.get("data", {}).get("issues", {}).get("nodes", [])

  def get_issue(self, issue_id: str) -> Optional[Dict[str, Any]]:
    """
    Get a specific issue by ID or identifier (e.g., "ENG-123").

    Args:
      issue_id: Issue ID or identifier

    Returns:
      Issue dictionary or None if not found
    """
    # Try as identifier first (e.g., "ENG-123")
    query = f"""
    query {{
      issue(id: "{issue_id}") {{
        id
        identifier
        title
        description
        priority
        state {{
          id
          name
          type
        }}
        assignee {{
          id
          name
          email
        }}
        team {{
          id
          name
          key
        }}
        createdAt
        updatedAt
        dueDate
        labels {{
          nodes {{
            id
            name
            color
          }}
        }}
      }}
    }}
    """
    try:
      data = self._query(query)
      return data.get("data", {}).get("issue")
    except RuntimeError:
      # Try searching by identifier if direct lookup fails
      return self._search_issue_by_identifier(issue_id)

  def _search_issue_by_identifier(self, identifier: str) -> Optional[Dict[str, Any]]:
    """Search for issue by identifier (e.g., "ENG-123")."""
    query = f"""
    query {{
      issues(
        filter: {{ identifier: {{ eq: "{identifier}" }} }},
        first: 1
      ) {{
        nodes {{
          id
          identifier
          title
          description
          priority
          state {{
            id
            name
            type
          }}
          assignee {{
            id
            name
            email
          }}
          team {{
            id
            name
            key
          }}
          createdAt
          updatedAt
          dueDate
        }}
      }}
    }}
    """
    data = self._query(query)
    issues = data.get("data", {}).get("issues", {}).get("nodes", [])
    return issues[0] if issues else None

  def create_issue(
    self,
    team_id: str,
    title: str,
    description: Optional[str] = None,
    assignee_id: Optional[str] = None,
    state_id: Optional[str] = None,
    priority: Optional[int] = None,
    label_ids: Optional[List[str]] = None,
  ) -> Dict[str, Any]:
    """
    Create a new issue.

    Args:
      team_id: Team ID where issue will be created
      title: Issue title
      description: Optional issue description
      assignee_id: Optional assignee user ID
      state_id: Optional initial state ID
      priority: Optional priority (0-4, where 0 is urgent)
      label_ids: Optional list of label IDs

    Returns:
      Created issue dictionary
    """
    variables: Dict[str, Any] = {
      "teamId": team_id,
      "title": title,
    }

    if description:
      variables["description"] = description
    if assignee_id:
      variables["assigneeId"] = assignee_id
    if state_id:
      variables["stateId"] = state_id
    if priority is not None:
      variables["priority"] = priority
    if label_ids:
      variables["labelIds"] = label_ids

    query = """
    mutation CreateIssue(
      $teamId: String!,
      $title: String!,
      $description: String,
      $assigneeId: String,
      $stateId: String,
      $priority: Int,
      $labelIds: [String!]
    ) {
      issueCreate(
        input: {
          teamId: $teamId
          title: $title
          description: $description
          assigneeId: $assigneeId
          stateId: $stateId
          priority: $priority
          labelIds: $labelIds
        }
      ) {
        success
        issue {
          id
          identifier
          title
          description
          state {
            id
            name
          }
          assignee {
            id
            name
          }
          team {
            id
            name
            key
          }
        }
      }
    }
    """

    data = self._query(query, variables)
    result = data.get("data", {}).get("issueCreate", {})
    if not result.get("success"):
      raise RuntimeError("Failed to create issue in Linear")

    return result.get("issue", {})

  def update_issue(
    self,
    issue_id: str,
    title: Optional[str] = None,
    description: Optional[str] = None,
    assignee_id: Optional[str] = None,
    state_id: Optional[str] = None,
    priority: Optional[int] = None,
    label_ids: Optional[List[str]] = None,
  ) -> Dict[str, Any]:
    """
    Update an existing issue.

    Args:
      issue_id: Issue ID to update
      title: Optional new title
      description: Optional new description
      assignee_id: Optional new assignee (use empty string to unassign)
      state_id: Optional new state ID
      priority: Optional priority (0-4)
      label_ids: Optional list of label IDs

    Returns:
      Updated issue dictionary
    """
    variables: Dict[str, Any] = {"id": issue_id}

    if title:
      variables["title"] = title
    if description is not None:
      variables["description"] = description
    if assignee_id is not None:
      variables["assigneeId"] = assignee_id
    if state_id:
      variables["stateId"] = state_id
    if priority is not None:
      variables["priority"] = priority
    if label_ids is not None:
      variables["labelIds"] = label_ids

    query = """
    mutation UpdateIssue(
      $id: String!,
      $title: String,
      $description: String,
      $assigneeId: String,
      $stateId: String,
      $priority: Int,
      $labelIds: [String!]
    ) {
      issueUpdate(
        id: $id,
        input: {
          title: $title
          description: $description
          assigneeId: $assigneeId
          stateId: $stateId
          priority: $priority
          labelIds: $labelIds
        }
      ) {
        success
        issue {
          id
          identifier
          title
          description
          state {
            id
            name
          }
          assignee {
            id
            name
          }
        }
      }
    }
    """

    data = self._query(query, variables)
    result = data.get("data", {}).get("issueUpdate", {})
    if not result.get("success"):
      raise RuntimeError("Failed to update issue in Linear")

    return result.get("issue", {})

  def add_comment(self, issue_id: str, body: str) -> Dict[str, Any]:
    """
    Add a comment to an issue.

    Args:
      issue_id: Issue ID
      body: Comment text

    Returns:
      Created comment dictionary
    """
    query = """
    mutation CreateComment($issueId: String!, $body: String!) {
      commentCreate(
        input: {
          issueId: $issueId
          body: $body
        }
      ) {
        success
        comment {
          id
          body
          createdAt
          user {
            id
            name
          }
        }
      }
    }
    """

    data = self._query(query, {"issueId": issue_id, "body": body})
    result = data.get("data", {}).get("commentCreate", {})
    if not result.get("success"):
      raise RuntimeError("Failed to create comment in Linear")

    return result.get("comment", {})

  def get_states(self, team_id: Optional[str] = None) -> List[Dict[str, Any]]:
    """
    Get workflow states (e.g., "Backlog", "In Progress", "Done").

    Args:
      team_id: Optional team ID to filter states

    Returns:
      List of state dictionaries
    """
    if team_id:
      query = f"""
      query {{
        team(id: "{team_id}") {{
          states {{
            nodes {{
              id
              name
              type
              position
            }}
          }}
        }}
      }}
      """
      data = self._query(query)
      return data.get("data", {}).get("team", {}).get("states", {}).get("nodes", [])
    else:
      # Get states from all teams
      query = """
      query {
        workflowStates {
          nodes {
            id
            name
            type
            position
            team {
              id
              name
            }
          }
        }
      }
      """
      data = self._query(query)
      return data.get("data", {}).get("workflowStates", {}).get("nodes", [])
