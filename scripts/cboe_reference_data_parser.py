#!/usr/bin/env python3
"""
CBOE Reference Data Parser

Parses CBOE Options Reference Data to extract QSB (Quoted Spread Book) instruments.
This script helps explore what's available for free from CBOE.

Usage:
    python scripts/cboe_reference_data_parser.py [--output-dir docs/cboe_data]
"""

import argparse
import json
import logging
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional
from urllib.parse import urljoin

import requests
from bs4 import BeautifulSoup

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


class CBOEReferenceDataParser:
    """Parser for CBOE Options Reference Data."""

    # CBOE Reference Data URLs (to be discovered/confirmed)
    BASE_URLS = [
        "https://www.cboe.com/us/options/reference_data/",
        "https://www.cboe.com/us/options/market_data/",
        "https://www.cboe.com/us/options/",
    ]

    def __init__(self, output_dir: Path = Path("docs/cboe_data")):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.session = requests.Session()
        self.session.headers.update(
            {
                "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36"
            }
        )

    def discover_reference_data_urls(self) -> List[str]:
        """
        Discover CBOE reference data URLs.

        Returns:
            List of discovered URLs
        """
        logger.info("Discovering CBOE reference data URLs...")
        discovered_urls = []

        for base_url in self.BASE_URLS:
            try:
                response = self.session.get(base_url, timeout=10)
                response.raise_for_status()

                soup = BeautifulSoup(response.content, "html.parser")

                # Look for links to reference data, JSON, or QSB
                for link in soup.find_all("a", href=True):
                    href = link.get("href")
                    text = link.get_text().lower()

                    if any(
                        keyword in text or keyword in href.lower()
                        for keyword in ["reference", "qsb", "json", "spread", "complex"]
                    ):
                        full_url = urljoin(base_url, href)
                        discovered_urls.append(full_url)
                        logger.info(f"Found potential URL: {full_url}")

            except requests.RequestException as e:
                logger.warning(f"Failed to access {base_url}: {e}")

        return discovered_urls

    def fetch_json_data(self, url: str) -> Optional[Dict]:
        """
        Fetch and parse JSON data from URL.

        Args:
            url: URL to fetch

        Returns:
            Parsed JSON data or None
        """
        try:
            response = self.session.get(url, timeout=10)
            response.raise_for_status()

            # Try to parse as JSON
            try:
                return response.json()
            except json.JSONDecodeError:
                logger.warning(f"URL {url} is not JSON, trying HTML parse...")
                return None

        except requests.RequestException as e:
            logger.error(f"Failed to fetch {url}: {e}")
            return None

    def parse_html_for_data(self, url: str) -> Optional[Dict]:
        """
        Parse HTML page for data tables or embedded JSON.

        Args:
            url: URL to parse

        Returns:
            Extracted data or None
        """
        try:
            response = self.session.get(url, timeout=10)
            response.raise_for_status()

            soup = BeautifulSoup(response.content, "html.parser")

            # Look for embedded JSON in script tags
            for script in soup.find_all("script"):
                if script.string:
                    # Try to find JSON data
                    if "qsb" in script.string.lower() or "spread" in script.string.lower():
                        logger.info(f"Found potential JSON in script tag on {url}")
                        # Try to extract JSON
                        # (This is a simplified version - may need more sophisticated parsing)

            # Look for data tables
            tables = soup.find_all("table")
            if tables:
                logger.info(f"Found {len(tables)} tables on {url}")
                # Extract table data (simplified - may need more work)

            return None

        except requests.RequestException as e:
            logger.error(f"Failed to parse HTML from {url}: {e}")
            return None

    def extract_qsb_instruments(self, data: Dict) -> List[Dict]:
        """
        Extract QSB (Quoted Spread Book) instruments from data.

        Args:
            data: Parsed data dictionary

        Returns:
            List of QSB instruments
        """
        qsb_instruments = []

        # This is a placeholder - actual structure depends on CBOE data format
        # We'll need to adapt this based on what we discover

        def search_dict(obj, path=""):
            """Recursively search dictionary for QSB-related data."""
            if isinstance(obj, dict):
                for key, value in obj.items():
                    current_path = f"{path}.{key}" if path else key

                    # Look for QSB-related keys
                    if any(
                        keyword in key.lower()
                        for keyword in ["qsb", "spread", "box", "complex", "cob"]
                    ):
                        logger.info(f"Found potential QSB data at: {current_path}")
                        qsb_instruments.append({"path": current_path, "data": value})

                    # Recurse
                    if isinstance(value, (dict, list)):
                        search_dict(value, current_path)

            elif isinstance(obj, list):
                for i, item in enumerate(obj):
                    search_dict(item, f"{path}[{i}]")

        search_dict(data)

        return qsb_instruments

    def save_data(self, data: Dict, filename: str):
        """
        Save data to file.

        Args:
            data: Data to save
            filename: Output filename
        """
        output_path = self.output_dir / filename
        with open(output_path, "w") as f:
            json.dump(data, f, indent=2)
        logger.info(f"Saved data to {output_path}")

    def run(self):
        """Main execution method."""
        logger.info("Starting CBOE Reference Data exploration...")

        # Discover URLs
        urls = self.discover_reference_data_urls()
        logger.info(f"Discovered {len(urls)} potential URLs")

        # Try to fetch and parse data from each URL
        all_data = {}
        qsb_instruments = []

        for url in urls:
            logger.info(f"Processing {url}...")

            # Try JSON first
            json_data = self.fetch_json_data(url)
            if json_data:
                all_data[url] = json_data
                qsb_instruments.extend(self.extract_qsb_instruments(json_data))
                continue

            # Try HTML parsing
            html_data = self.parse_html_for_data(url)
            if html_data:
                all_data[url] = html_data
                qsb_instruments.extend(self.extract_qsb_instruments(html_data))

        # Save discovered data
        if all_data:
            self.save_data(
                all_data,
                f"cboe_reference_data_{datetime.now().strftime('%Y%m%d')}.json",
            )

        # Save QSB instruments
        if qsb_instruments:
            self.save_data(
                {"qsb_instruments": qsb_instruments, "count": len(qsb_instruments)},
                f"cboe_qsb_instruments_{datetime.now().strftime('%Y%m%d')}.json",
            )
            logger.info(f"Found {len(qsb_instruments)} QSB-related data points")

        # Generate summary report
        self.generate_summary_report(urls, all_data, qsb_instruments)

    def generate_summary_report(
        self, urls: List[str], data: Dict, qsb_instruments: List[Dict]
    ):
        """Generate a summary report of findings."""
        report_path = self.output_dir / f"exploration_report_{datetime.now().strftime('%Y%m%d')}.md"

        with open(report_path, "w") as f:
            f.write("# CBOE Reference Data Exploration Report\n\n")
            f.write(f"**Date**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
            f.write("## Discovered URLs\n\n")
            for url in urls:
                f.write(f"- {url}\n")
            f.write("\n## Data Retrieved\n\n")
            f.write(f"- URLs processed: {len(urls)}\n")
            f.write(f"- URLs with data: {len(data)}\n")
            f.write(f"- QSB-related data points: {len(qsb_instruments)}\n")
            f.write("\n## QSB Instruments Found\n\n")
            for instrument in qsb_instruments[:20]:  # First 20
                f.write(f"- {instrument.get('path', 'unknown')}\n")
            if len(qsb_instruments) > 20:
                f.write(f"\n... and {len(qsb_instruments) - 20} more\n")

        logger.info(f"Generated report: {report_path}")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Parse CBOE Reference Data")
    parser.add_argument(
        "--output-dir",
        type=str,
        default="docs/cboe_data",
        help="Output directory for parsed data",
    )
    args = parser.parse_args()

    parser_instance = CBOEReferenceDataParser(output_dir=args.output_dir)
    parser_instance.run()


if __name__ == "__main__":
    main()
