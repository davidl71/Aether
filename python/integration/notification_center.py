"""
notification_center.py - Routing layer for strategy event notifications.
Supports email, webhook, SMS (Twilio), and Telegram delivery inspired by Lean CLI.
"""
from __future__ import annotations

import json
import logging
import smtplib
import ssl
from email.mime.text import MIMEText
from typing import Any, Dict, Iterable, List, Optional

import requests

logger = logging.getLogger(__name__)


class NotificationChannel:
    """Base notification channel."""

    def __init__(self, config: Dict[str, Any], subscribed_events: Optional[Iterable[str]] = None) -> None:
        self.config = config
        self.subscribed_events = set(e.lower() for e in (subscribed_events or []))

    def accepts_event(self, event_type: str) -> bool:
        if not self.subscribed_events:
            return True
        return event_type.lower() in self.subscribed_events

    def send(self, event_type: str, severity: str, title: str, message: str, payload: Optional[Dict[str, Any]] = None) -> None:
        raise NotImplementedError


class EmailChannel(NotificationChannel):
    def send(self, event_type: str, severity: str, title: str, message: str, payload: Optional[Dict[str, Any]] = None) -> None:
        if not self.accepts_event(event_type):
            return

        smtp_host = self.config.get("smtp_host")
        smtp_port = int(self.config.get("smtp_port", 587))
        username = self.config.get("username")
        password = self.config.get("password")
        sender = self.config.get("from")
        recipients = self.config.get("to", [])

        if not smtp_host or not sender or not recipients:
            logger.warning("Email channel missing smtp_host/from/to configuration")
            return

        mime = MIMEText(message, "plain", "utf-8")
        mime["Subject"] = title
        mime["From"] = sender
        mime["To"] = ", ".join(recipients)

        use_tls = bool(self.config.get("use_tls", True))

        try:
            context = ssl.create_default_context()
            with smtplib.SMTP(smtp_host, smtp_port, timeout=10) as server:
                if use_tls:
                    server.starttls(context=context)
                if username and password:
                    server.login(username, password)
                server.sendmail(sender, recipients, mime.as_string())
            logger.info("Notification email delivered to %s", recipients)
        except Exception as exc:  # pragma: no cover - network interaction
            logger.error("Failed to send notification email: %s", exc)


class WebhookChannel(NotificationChannel):
    def send(self, event_type: str, severity: str, title: str, message: str, payload: Optional[Dict[str, Any]] = None) -> None:
        if not self.accepts_event(event_type):
            return

        url = self.config.get("url")
        if not url:
            logger.warning("Webhook channel missing url configuration")
            return

        data = {
            "event": event_type,
            "severity": severity,
            "title": title,
            "message": message,
            "payload": payload or {},
        }

        headers = {"Content-Type": "application/json"}
        headers.update(self.config.get("headers", {}))

        try:
            response = requests.post(url, headers=headers, data=json.dumps(data), timeout=5)
            response.raise_for_status()
            logger.info("Notification webhook delivered to %s", url)
        except Exception as exc:  # pragma: no cover - network interaction
            logger.error("Failed to send webhook notification: %s", exc)


class TelegramChannel(NotificationChannel):
    def send(self, event_type: str, severity: str, title: str, message: str, payload: Optional[Dict[str, Any]] = None) -> None:
        if not self.accepts_event(event_type):
            return

        bot_token = self.config.get("bot_token")
        chat_id = self.config.get("chat_id")

        if not bot_token or not chat_id:
            logger.warning("Telegram channel missing bot_token/chat_id configuration")
            return

        url = f"https://api.telegram.org/bot{bot_token}/sendMessage"
        text = f"*{title}*\nSeverity: {severity.upper()}\n{message}"

        data = {
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown",
        }

        try:
            response = requests.post(url, data=data, timeout=5)
            response.raise_for_status()
            logger.info("Notification delivered via Telegram")
        except Exception as exc:  # pragma: no cover - network interaction
            logger.error("Failed to send Telegram notification: %s", exc)


class SMSChannel(NotificationChannel):
    def send(self, event_type: str, severity: str, title: str, message: str, payload: Optional[Dict[str, Any]] = None) -> None:
        if not self.accepts_event(event_type):
            return

        account_sid = self.config.get("twilio_account_sid")
        auth_token = self.config.get("twilio_auth_token")
        from_number = self.config.get("from")
        to_numbers = self.config.get("to", [])

        if not account_sid or not auth_token or not from_number or not to_numbers:
            logger.warning("SMS channel missing Twilio configuration (account_sid/auth_token/from/to)")
            return

        url = f"https://api.twilio.com/2010-04-01/Accounts/{account_sid}/Messages.json"

        for recipient in to_numbers:
            payload_data = {
                "From": from_number,
                "To": recipient,
                "Body": f"{title} [{severity.upper()}]: {message}",
            }

            try:
                response = requests.post(
                    url,
                    data=payload_data,
                    auth=(account_sid, auth_token),
                    timeout=5,
                )
                response.raise_for_status()
                logger.info("Notification SMS delivered to %s", recipient)
            except Exception as exc:  # pragma: no cover - network interaction
                logger.error("Failed to send SMS notification to %s: %s", recipient, exc)


CHANNEL_FACTORIES = {
    "email": EmailChannel,
    "webhook": WebhookChannel,
    "sms": SMSChannel,
    "telegram": TelegramChannel,
}


class NotificationCenter:
    def __init__(self, config: Optional[Dict[str, Any]] = None) -> None:
        config = config or {}
        self.enabled = bool(config.get("enabled", False))
        self.default_severity = config.get("default_severity", "info")
        self.channels: List[NotificationChannel] = []

        if not self.enabled:
            logger.info("Notifications disabled")
            return

        for channel_cfg in config.get("channels", []):
            channel_type = channel_cfg.get("type", "").lower()
            events = channel_cfg.get("events", [])
            factory = CHANNEL_FACTORIES.get(channel_type)
            if not factory:
                logger.warning("Unknown notification channel type: %s", channel_type)
                continue
            channel = factory(channel_cfg, events)
            self.channels.append(channel)

        if not self.channels:
            logger.warning("Notifications enabled but no valid channels configured")

    def notify(
        self,
        event_type: str,
        title: str,
        message: str,
        severity: Optional[str] = None,
        payload: Optional[Dict[str, Any]] = None,
    ) -> None:
        if not self.enabled or not self.channels:
            return

        severity = (severity or self.default_severity).lower()

        for channel in self.channels:
            try:
                channel.send(event_type, severity, title, message, payload)
            except Exception as exc:  # pragma: no cover - defensive
                logger.error("Notification channel failed: %s", exc)


