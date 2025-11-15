#!/usr/bin/env python3
"""
LiveVol API Explorer

Explores LiveVol API during trial to discover quoted spread capabilities.
Tests endpoints for box spreads, QSB instruments, and strategy quotes.

Usage:
    python scripts/livevol_api_explorer.py \
        --client-id YOUR_CLIENT_ID \
        --client-secret YOUR_CLIENT_SECRET \
        --output-dir docs/livevol_exploration
"""

import argparse
import base64
import json
import logging
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

import requests

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


class LiveVolAPIExplorer:
    """Explorer for LiveVol API to discover quoted spread capabilities."""

    BASE_URL = "https://api.livevol.com/v1"
    IDENTITY_URL = "https://id.livevol.com"  # Authentication server
    DOCS_URL = "https://api.livevol.com/v1/docs/"

    def __init__(
        self,
        client_id: str,
        client_secret: str,
        output_dir: Path = Path("docs/livevol_exploration"),
    ):
        self.client_id = client_id
        self.client_secret = client_secret
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.session = requests.Session()
        self.access_token: Optional[str] = None

    def authenticate(self) -> bool:
        """
        Authenticate with LiveVol API using OAuth 2.0 Client Credentials flow.

        Uses Basic Auth with base64 encoded client_id:client_secret.
        See: https://api.livevol.com/v1/docs/Home/Authentication

        Returns:
            True if authentication successful
        """
        logger.info("Authenticating with LiveVol API...")
        logger.info(f"Using identity server: {self.IDENTITY_URL}")

        # OAuth 2.0 token endpoint (Client Credentials flow)
        token_url = f"{self.IDENTITY_URL}/connect/token"

        # Create Basic Auth header with base64 encoded client_id:client_secret
        credentials = f"{self.client_id}:{self.client_secret}"
        encoded_credentials = base64.b64encode(credentials.encode()).decode()
        auth_header = f"Basic {encoded_credentials}"

        try:
            response = self.session.post(
                token_url,
                data={"grant_type": "client_credentials"},
                headers={
                    "Authorization": auth_header,
                    "Content-Type": "application/x-www-form-urlencoded",
                },
                timeout=10,
            )
            response.raise_for_status()

            token_data = response.json()
            self.access_token = token_data.get("access_token")
            expires_in = token_data.get("expires_in", 3600)
            token_type = token_data.get("token_type", "Bearer")

            if self.access_token:
                self.session.headers.update(
                    {"Authorization": f"{token_type} {self.access_token}"}
                )
                logger.info(f"Authentication successful (expires in {expires_in}s)")
                return True
            else:
                logger.error("No access token in response")
                logger.error(f"Response: {token_data}")
                return False

        except requests.RequestException as e:
            logger.error(f"Authentication failed: {e}")
            if hasattr(e, "response") and e.response is not None:
                try:
                    error_data = e.response.json()
                    logger.error(f"Error response: {error_data}")
                except:
                    logger.error(f"Error response: {e.response.text}")
            logger.info("Verify credentials are correct and trial includes API access")
            return False

    def fetch_api_docs(self) -> Optional[Dict]:
        """
        Fetch API documentation to discover endpoints.

        Returns:
            API documentation or None
        """
        logger.info("Fetching API documentation...")

        try:
            # Try to fetch docs (may be HTML or JSON)
            response = self.session.get(self.DOCS_URL, timeout=10)
            response.raise_for_status()

            # Try JSON first
            try:
                return response.json()
            except json.JSONDecodeError:
                # If HTML, we'll need to parse it
                logger.info("API docs are HTML format - will need to parse")
                return {"html": response.text[:1000]}  # First 1000 chars

        except requests.RequestException as e:
            logger.error(f"Failed to fetch API docs: {e}")
            return None

    def discover_endpoints(self) -> List[str]:
        """
        Discover API endpoints by testing common patterns.

        Returns:
            List of discovered endpoints
        """
        logger.info("Discovering API endpoints...")

        # Common endpoint patterns to test
        endpoint_patterns = [
            # Strategy/Spread endpoints
            "/strategy/quotes",
            "/strategy/box-spread",
            "/strategy/spread",
            "/complex/quotes",
            "/complex/spread",
            "/qsb/instruments",
            "/qsb/quotes",
            "/spread/quotes",
            "/box-spread/quotes",
            # Options endpoints
            "/options/quotes",
            "/options/chains",
            "/options/time-and-sales",
            # Strategy scan endpoints
            "/strategy/scan",
            "/scan/strategy",
        ]

        discovered_endpoints = []

        for pattern in endpoint_patterns:
            endpoint = f"{self.BASE_URL}{pattern}"
            logger.info(f"Testing endpoint: {endpoint}")

            try:
                response = self.session.get(endpoint, timeout=10)

                if response.status_code == 200:
                    logger.info(f"✅ Found endpoint: {pattern}")
                    discovered_endpoints.append(pattern)
                elif response.status_code == 401:
                    logger.warning(f"⚠️  Endpoint exists but requires auth: {pattern}")
                    discovered_endpoints.append(f"{pattern} (auth required)")
                elif response.status_code == 404:
                    logger.debug(f"❌ Endpoint not found: {pattern}")
                else:
                    logger.info(
                        f"⚠️  Endpoint returned {response.status_code}: {pattern}"
                    )
                    discovered_endpoints.append(f"{pattern} (status {response.status_code})")

            except requests.RequestException as e:
                logger.debug(f"Error testing {pattern}: {e}")

        return discovered_endpoints

    def test_quoted_spread_endpoints(self) -> Dict:
        """
        Test endpoints that might provide quoted spreads.

        Returns:
            Dictionary of test results
        """
        logger.info("Testing quoted spread endpoints...")

        test_results = {}

        # Test endpoints with SPX box spread parameters
        test_cases = [
            {
                "name": "strategy_quotes",
                "endpoint": "/strategy/quotes",
                "params": {
                    "symbol": "SPX",
                    "strategy_type": "box_spread",
                    "strike_low": 4000,
                    "strike_high": 5000,
                },
            },
            {
                "name": "complex_quotes",
                "endpoint": "/complex/quotes",
                "params": {
                    "symbol": "SPX",
                    "legs": "4",  # Box spread has 4 legs
                },
            },
            {
                "name": "qsb_quotes",
                "endpoint": "/qsb/quotes",
                "params": {
                    "symbol": "SPX",
                },
            },
        ]

        for test_case in test_cases:
            endpoint = f"{self.BASE_URL}{test_case['endpoint']}"
            logger.info(f"Testing {test_case['name']}: {endpoint}")

            try:
                response = self.session.get(
                    endpoint, params=test_case["params"], timeout=10
                )

                test_results[test_case["name"]] = {
                    "endpoint": test_case["endpoint"],
                    "status_code": response.status_code,
                    "response": response.text[:500] if response.text else None,
                    "success": response.status_code == 200,
                }

                if response.status_code == 200:
                    try:
                        test_results[test_case["name"]]["data"] = response.json()
                    except json.JSONDecodeError:
                        test_results[test_case["name"]]["data"] = None

            except requests.RequestException as e:
                test_results[test_case["name"]] = {
                    "endpoint": test_case["endpoint"],
                    "error": str(e),
                    "success": False,
                }

        return test_results

    def test_qsb_instruments(self) -> Dict:
        """
        Test QSB instrument discovery endpoints.

        Returns:
            Dictionary of test results
        """
        logger.info("Testing QSB instrument endpoints...")

        test_results = {}

        qsb_endpoints = [
            "/qsb/instruments",
            "/qsb/list",
            "/complex/qsb",
            "/reference/qsb",
        ]

        for endpoint_path in qsb_endpoints:
            endpoint = f"{self.BASE_URL}{endpoint_path}"
            logger.info(f"Testing QSB endpoint: {endpoint}")

            try:
                response = self.session.get(endpoint, timeout=10)

                test_results[endpoint_path] = {
                    "status_code": response.status_code,
                    "success": response.status_code == 200,
                }

                if response.status_code == 200:
                    try:
                        test_results[endpoint_path]["data"] = response.json()
                    except json.JSONDecodeError:
                        test_results[endpoint_path]["data"] = response.text[:1000]

            except requests.RequestException as e:
                test_results[endpoint_path] = {
                    "error": str(e),
                    "success": False,
                }

        return test_results

    def save_results(self, results: Dict, filename: str):
        """Save exploration results to file."""
        output_path = self.output_dir / filename
        with open(output_path, "w") as f:
            json.dump(results, f, indent=2)
        logger.info(f"Saved results to {output_path}")

    def generate_report(self, all_results: Dict):
        """Generate a markdown report of findings."""
        report_path = self.output_dir / f"exploration_report_{datetime.now().strftime('%Y%m%d')}.md"

        with open(report_path, "w") as f:
            f.write("# LiveVol API Exploration Report\n\n")
            f.write(f"**Date**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
            f.write("## Authentication\n\n")
            f.write(f"- Status: {'✅ Success' if all_results.get('auth_success') else '❌ Failed'}\n")
            f.write(f"- Access Token: {'Present' if all_results.get('access_token') else 'Missing'}\n\n")
            f.write("## Discovered Endpoints\n\n")
            for endpoint in all_results.get("discovered_endpoints", []):
                f.write(f"- {endpoint}\n")
            f.write("\n## Quoted Spread Tests\n\n")
            for test_name, result in all_results.get("quoted_spread_tests", {}).items():
                status = "✅" if result.get("success") else "❌"
                f.write(f"- {status} {test_name}: {result.get('endpoint', 'unknown')}\n")
            f.write("\n## QSB Instrument Tests\n\n")
            for endpoint, result in all_results.get("qsb_tests", {}).items():
                status = "✅" if result.get("success") else "❌"
                f.write(f"- {status} {endpoint}\n")
            f.write("\n## Recommendations\n\n")
            f.write("Based on the exploration results:\n")
            if all_results.get("quoted_spread_tests"):
                f.write("- Document which endpoints support quoted spreads\n")
            if all_results.get("qsb_tests"):
                f.write("- Document QSB instrument access methods\n")
            f.write("- Evaluate integration feasibility\n")

        logger.info(f"Generated report: {report_path}")

    def run(self):
        """Main execution method."""
        logger.info("Starting LiveVol API exploration...")

        all_results = {}

        # Authenticate
        auth_success = self.authenticate()
        all_results["auth_success"] = auth_success
        all_results["access_token"] = bool(self.access_token)

        # Fetch API docs
        api_docs = self.fetch_api_docs()
        if api_docs:
            all_results["api_docs"] = api_docs
            self.save_results(api_docs, "api_docs.json")

        # Discover endpoints
        discovered_endpoints = self.discover_endpoints()
        all_results["discovered_endpoints"] = discovered_endpoints
        logger.info(f"Discovered {len(discovered_endpoints)} endpoints")

        # Test quoted spread endpoints
        if auth_success:
            quoted_spread_tests = self.test_quoted_spread_endpoints()
            all_results["quoted_spread_tests"] = quoted_spread_tests
            self.save_results(quoted_spread_tests, "quoted_spread_tests.json")

            # Test QSB instruments
            qsb_tests = self.test_qsb_instruments()
            all_results["qsb_tests"] = qsb_tests
            self.save_results(qsb_tests, "qsb_tests.json")
        else:
            logger.warning("Skipping endpoint tests - authentication failed")

        # Save all results
        self.save_results(all_results, "exploration_results.json")

        # Generate report
        self.generate_report(all_results)

        logger.info("Exploration complete!")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Explore LiveVol API for quoted spreads")
    parser.add_argument(
        "--client-id", required=True, help="LiveVol OAuth 2.0 client ID"
    )
    parser.add_argument(
        "--client-secret", required=True, help="LiveVol OAuth 2.0 client secret"
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default="docs/livevol_exploration",
        help="Output directory for exploration results",
    )
    args = parser.parse_args()

    explorer = LiveVolAPIExplorer(
        client_id=args.client_id,
        client_secret=args.client_secret,
        output_dir=args.output_dir,
    )
    explorer.run()


if __name__ == "__main__":
    main()
