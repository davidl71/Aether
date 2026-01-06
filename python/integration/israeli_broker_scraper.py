"""
israeli_broker_scraper.py - Web scraping client for Israeli broker position extraction

Uses Playwright for browser automation to extract position data from Israeli broker
web portals that don't provide APIs or export functionality.

Based on research completed in T-147 (Research web scraping frameworks).
"""

from __future__ import annotations

import logging
import os
import time
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

# Optional Playwright import (graceful degradation)
try:
    from playwright.sync_api import Browser, BrowserContext, Page, sync_playwright, TimeoutError as PlaywrightTimeoutError
    PLAYWRIGHT_AVAILABLE = True
except ImportError:
    Browser = None
    BrowserContext = None
    Page = None
    sync_playwright = None
    PlaywrightTimeoutError = Exception
    PLAYWRIGHT_AVAILABLE = False

from services.security import RateLimiter

logger = logging.getLogger(__name__)


class IsraeliBrokerScraperError(RuntimeError):
    """Generic error raised for web scraping failures."""


@dataclass
class Position:
    """Standardized position data model for Israeli broker positions."""
    symbol: str
    quantity: float
    cost_basis: float
    current_price: float
    currency: str  # USD, ILS, etc.
    broker: str
    account_id: Optional[str] = None
    last_updated: Optional[datetime] = None
    unrealized_pnl: Optional[float] = None

    # TASE-specific fields
    exchange: Optional[str] = None  # "TASE", "NYSE", etc.
    instrument_type: Optional[str] = None  # "stock", "option", "future", "bond", "etf"

    def calculate_pnl(self) -> float:
        """Calculate unrealized P&L."""
        return (self.current_price - self.cost_basis) * self.quantity


class IsraeliBrokerWebScraper:
    """
    Web scraper for Israeli broker position extraction using Playwright.

    Supports login automation, position extraction, session management, and
    error handling with screenshot on failure.
    """

    def __init__(
        self,
        broker: str,
        login_url: str,
        positions_url: str,
        username: Optional[str] = None,
        password: Optional[str] = None,
        headless: bool = True,
        rate_limiter: Optional[RateLimiter] = None,
        screenshot_dir: Optional[Path] = None,
        timeout_seconds: int = 30,
    ) -> None:
        """
        Initialize web scraper.

        Args:
            broker: Broker name identifier
            login_url: URL for login page
            positions_url: URL for positions page
            username: Login username (or use environment variable)
            password: Login password (or use environment variable)
            headless: Run browser in headless mode
            rate_limiter: Optional RateLimiter instance for rate limiting
            screenshot_dir: Directory for saving failure screenshots
            timeout_seconds: Default timeout for page operations
        """
        self.broker = broker
        self.login_url = login_url
        self.positions_url = positions_url
        self.headless = headless
        self.timeout = timeout_seconds * 1000  # Playwright uses milliseconds

        # Credentials (from parameters or environment)
        self.username = username or os.getenv(f"{broker.upper()}_USERNAME", "")
        self.password = password or os.getenv(f"{broker.upper()}_PASSWORD", "")

        if not self.username or not self.password:
            raise IsraeliBrokerScraperError(
                f"Missing credentials for {broker}. Provide username/password or set "
                f"{broker.upper()}_USERNAME and {broker.upper()}_PASSWORD environment variables."
            )

        # Rate limiting
        self.rate_limiter = rate_limiter or RateLimiter(requests_per_minute=60, requests_per_second=10)

        # Screenshot directory for error debugging
        self.screenshot_dir = screenshot_dir or Path("logs/screenshots")
        self.screenshot_dir.mkdir(parents=True, exist_ok=True)

        # Playwright browser context
        self._playwright = None
        self._browser: Optional[Browser] = None
        self._context: Optional[BrowserContext] = None
        self._page: Optional[Page] = None
        self._connected = False

        # Session management
        self._cookies: List[Dict[str, Any]] = []

    def _ensure_playwright(self) -> None:
        """Ensure Playwright is available."""
        if not PLAYWRIGHT_AVAILABLE:
            raise IsraeliBrokerScraperError(
                "Playwright not available. Install with: pip install playwright && playwright install chromium"
            )

    def _check_rate_limit(self) -> None:
        """Check rate limit before making request."""
        if not self.rate_limiter.check_rate_limit(self.broker):
            wait_time = 60.0 / self.rate_limiter.requests_per_minute
            logger.warning(f"Rate limit reached for {self.broker}, waiting {wait_time:.1f}s")
            time.sleep(wait_time)

    def _take_screenshot(self, filename: str) -> Path:
        """Take screenshot for debugging."""
        if self._page:
            screenshot_path = self.screenshot_dir / f"{self.broker}_{filename}_{int(time.time())}.png"
            try:
                self._page.screenshot(path=str(screenshot_path))
                logger.info(f"Screenshot saved to {screenshot_path}")
                return screenshot_path
            except Exception as e:
                logger.error(f"Failed to take screenshot: {e}")
        return Path()

    def connect(self) -> bool:
        """
        Initialize browser and create context.

        Returns:
            True if connected successfully, False otherwise
        """
        self._ensure_playwright()

        try:
            self._playwright = sync_playwright().start()
            self._browser = self._playwright.chromium.launch(headless=self.headless)
            self._context = self._browser.new_context(
                viewport={"width": 1920, "height": 1080},
                user_agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36"
            )
            self._page = self._context.new_page()
            self._connected = True
            logger.info(f"Browser connected for {self.broker}")
            return True
        except Exception as e:
            logger.error(f"Failed to connect browser for {self.broker}: {e}")
            self._connected = False
            return False

    def login(self) -> bool:
        """
        Login to broker web portal.

        Returns:
            True if login successful, False otherwise
        """
        if not self._connected:
            if not self.connect():
                return False

        try:
            self._check_rate_limit()

            # Navigate to login page
            logger.info(f"Navigating to login page: {self.login_url}")
            self._page.goto(self.login_url, timeout=self.timeout, wait_until="domcontentloaded")

            # Wait for login form and fill credentials
            # Generic selectors - should be overridden in broker-specific subclasses
            username_selector = self._get_username_selector()
            password_selector = self._get_password_selector()
            submit_selector = self._get_submit_selector()

            logger.info("Filling login credentials")
            self._page.fill(username_selector, self.username)
            self._page.fill(password_selector, self.password)

            # Submit form
            self._page.click(submit_selector)

            # Wait for navigation or success indicator
            success_selector = self._get_login_success_selector()
            try:
                self._page.wait_for_selector(success_selector, timeout=self.timeout)
                logger.info(f"Login successful for {self.broker}")

                # Save cookies for session reuse
                self._cookies = self._context.cookies()
                return True
            except PlaywrightTimeoutError:
                # Check for CAPTCHA or error messages
                if self._detect_captcha():
                    error_msg = f"CAPTCHA detected during login for {self.broker}"
                    logger.error(error_msg)
                    self._take_screenshot("login_captcha")
                    raise IsraeliBrokerScraperError(error_msg)

                # Check for error messages
                error_msg = self._detect_login_error()
                if error_msg:
                    logger.error(f"Login failed for {self.broker}: {error_msg}")
                    self._take_screenshot("login_error")
                    return False

                # Timeout - might be successful but slow
                logger.warning(f"Login timeout for {self.broker}, taking screenshot")
                self._take_screenshot("login_timeout")
                return False

        except Exception as e:
            logger.error(f"Login failed for {self.broker}: {e}")
            self._take_screenshot("login_exception")
            return False

    def _get_username_selector(self) -> str:
        """Get CSS selector for username field (override in subclasses)."""
        return 'input[name="username"], input[type="text"][id*="user"], input[id*="username"]'

    def _get_password_selector(self) -> str:
        """Get CSS selector for password field (override in subclasses)."""
        return 'input[name="password"], input[type="password"]'

    def _get_submit_selector(self) -> str:
        """Get CSS selector for submit button (override in subclasses)."""
        return 'button[type="submit"], input[type="submit"], button:has-text("Login"), button:has-text("כניסה")'

    def _get_login_success_selector(self) -> str:
        """Get CSS selector indicating successful login (override in subclasses)."""
        return '[id*="portfolio"], [id*="dashboard"], [class*="portfolio"], [class*="dashboard"]'

    def _detect_captcha(self) -> bool:
        """Detect if CAPTCHA is present on page."""
        if not self._page:
            return False
        captcha_selectors = [
            '[id*="captcha"]',
            '[class*="captcha"]',
            'iframe[src*="recaptcha"]',
            'iframe[src*="hcaptcha"]',
        ]
        for selector in captcha_selectors:
            try:
                if self._page.query_selector(selector):
                    return True
            except Exception:
                continue
        return False

    def _detect_login_error(self) -> Optional[str]:
        """Detect login error messages on page."""
        if not self._page:
            return None
        error_selectors = [
            '[class*="error"]',
            '[class*="alert"]',
            '[id*="error"]',
        ]
        for selector in error_selectors:
            try:
                element = self._page.query_selector(selector)
                if element:
                    text = element.inner_text()
                    if text and len(text) < 200:  # Reasonable error message length
                        return text.strip()
            except Exception:
                continue
        return None

    def scrape_positions(self) -> List[Position]:
        """
        Scrape position data from broker web portal.

        Returns:
            List of Position objects
        """
        if not self._connected:
            if not self.login():
                raise IsraeliBrokerScraperError(f"Not logged in to {self.broker}")

        try:
            self._check_rate_limit()

            # Navigate to positions page
            logger.info(f"Navigating to positions page: {self.positions_url}")
            self._page.goto(self.positions_url, timeout=self.timeout, wait_until="domcontentloaded")

            # Wait for positions table
            table_selector = self._get_positions_table_selector()
            try:
                self._page.wait_for_selector(table_selector, timeout=self.timeout)
            except PlaywrightTimeoutError:
                error_msg = f"Positions table not found for {self.broker}"
                logger.error(error_msg)
                self._take_screenshot("positions_table_not_found")
                raise IsraeliBrokerScraperError(error_msg)

            # Extract positions using JavaScript
            positions_data = self._page.evaluate(f"""
                () => {{
                    const table = document.querySelector('{table_selector}');
                    if (!table) return [];

                    const rows = Array.from(table.querySelectorAll('tr'));
                    return rows.slice(1).map(row => {{
                        const cells = row.querySelectorAll('td');
                        if (cells.length < 4) return null;

                        return {{
                            symbol: cells[0]?.textContent?.trim() || '',
                            quantity: parseFloat(cells[1]?.textContent?.replace(/[^0-9.-]/g, '') || '0'),
                            cost_basis: parseFloat(cells[2]?.textContent?.replace(/[^0-9.-]/g, '') || '0'),
                            current_price: parseFloat(cells[3]?.textContent?.replace(/[^0-9.-]/g, '') || '0'),
                            currency: cells[4]?.textContent?.trim() || 'ILS'
                        }};
                    }}).filter(p => p && p.symbol);
                }}
            """)

            # Convert to Position objects
            positions = []
            for data in positions_data:
                try:
                    position = Position(
                        symbol=data['symbol'],
                        quantity=data['quantity'],
                        cost_basis=data['cost_basis'],
                        current_price=data['current_price'],
                        currency=data.get('currency', 'ILS'),
                        broker=self.broker,
                        last_updated=datetime.now(),
                    )
                    position.unrealized_pnl = position.calculate_pnl()
                    positions.append(position)
                except (KeyError, ValueError) as e:
                    logger.warning(f"Failed to parse position data: {data}, error: {e}")
                    continue

            logger.info(f"Scraped {len(positions)} positions from {self.broker}")
            return positions

        except Exception as e:
            logger.error(f"Failed to scrape positions from {self.broker}: {e}")
            self._take_screenshot("scrape_positions_error")
            raise IsraeliBrokerScraperError(f"Failed to scrape positions: {e}") from e

    def _get_positions_table_selector(self) -> str:
        """Get CSS selector for positions table (override in subclasses)."""
        return 'table[id*="position"], table[class*="position"], table'

    def reconnect(self) -> bool:
        """Reconnect using saved cookies if available."""
        if not self._cookies:
            return self.login()

        try:
            if not self._connected:
                if not self.connect():
                    return False

            # Restore cookies
            self._context.add_cookies(self._cookies)
            logger.info(f"Reconnected to {self.broker} using saved cookies")
            return True
        except Exception as e:
            logger.warning(f"Failed to reconnect with cookies, trying fresh login: {e}")
            return self.login()

    def disconnect(self) -> None:
        """Disconnect and cleanup browser resources."""
        try:
            if self._page:
                self._page.close()
            if self._context:
                self._context.close()
            if self._browser:
                self._browser.close()
            if self._playwright:
                self._playwright.stop()

            self._page = None
            self._context = None
            self._browser = None
            self._playwright = None
            self._connected = False
            logger.info(f"Disconnected from {self.broker}")
        except Exception as e:
            logger.error(f"Error during disconnect: {e}")

    def __enter__(self):
        """Context manager entry."""
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.disconnect()
