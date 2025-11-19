"""
test_box_spread_basic.py - Basic test algorithm for LEAN box spread strategy

This is a simplified test algorithm to verify basic functionality before
running the full box spread strategy.
"""

from AlgorithmImports import *


class TestBoxSpreadBasic(QCAlgorithm):
    """
    Basic test algorithm to verify LEAN integration works.
    """

    def Initialize(self):
        """Initialize algorithm."""
        self.SetStartDate(2025, 1, 1)
        self.SetCash(100000)
        self.SetBenchmark("SPY")

        # Subscribe to SPY options
        option = self.AddOption("SPY")
        option.SetFilter(lambda u: u.Strikes(-10, +10).Expiration(0, 60))

        # Track data received
        self.data_count = 0
        self.option_chain_count = 0

        self.Log("Test algorithm initialized")

    def OnData(self, slice):
        """Process market data."""
        self.data_count += 1

        # Check for option chain
        option_chain = slice.OptionChains.get("SPY", None)
        if option_chain:
            self.option_chain_count += 1
            self.Log(f"Received option chain with {len(option_chain)} contracts")

            # Log first contract details
            if len(option_chain) > 0:
                contract = option_chain[0]
                self.Log(
                    f"Sample contract: {contract.Symbol} - Strike: {contract.Strike}, "
                    f"Bid: {contract.BidPrice}, Ask: {contract.AskPrice}"
                )

    def OnEndOfAlgorithm(self):
        """Log final statistics."""
        self.Log("=" * 50)
        self.Log("Test Results:")
        self.Log(f"  Data slices received: {self.data_count}")
        self.Log(f"  Option chains received: {self.option_chain_count}")
        self.Log("=" * 50)
