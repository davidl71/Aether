# Cash Flow Forecasting System

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Design Document

## Overview

This document designs a comprehensive future cash flow forecasting system that calculates and projects all future cash flows from the portfolio, including loan payments, option expirations, bond coupons/maturities, and dividends. This enables proactive cash flow planning and liquidity management.

## Integration with Investment Strategy Framework

**Purpose:** Forecast future cash flows to ensure sufficient liquidity for loan payments and optimize cash allocation based on upcoming cash inflows and outflows.

**Cash Flow Impact on Allocation:**

- **Loan Payments (Outflows):** Must ensure sufficient cash reserves for upcoming loan payments
- **Option Expirations (Inflows/Outflows):** Cash returned from long options, cash required for short options
- **Bond Coupons (Inflows):** Periodic coupon payments increase available cash
- **Bond Maturities (Inflows):** Principal returned at maturity increases available cash
- **Dividends (Inflows):** Dividend payments increase available cash

**Net Cash Flow Calculation:**

```
Net Cash Flow = Bond Coupons + Bond Maturities + Dividends + Option Expirations - Loan Payments
```

## Cash Flow Types

### 1. Loan Payments (Cash Outflows)

**Variable SHIR-Based Loans (Israel):**

- **Payment Frequency:** Monthly
- **Payment Amount:** Variable (changes with SHIR rate)
- **Calculation:** Principal × (SHIR + spread) / 12 months
- **Projection:** Use current SHIR rate, adjust monthly when SHIR changes
- **Currency:** ILS (convert to USD for unified view)

**Fixed Rate CPI-Linked Loans (Israel):**

- **Payment Frequency:** Monthly
- **Payment Amount:** Fixed interest payment + variable principal adjustment (CPI-linked)
- **Calculation:**
  - Interest payment: Fixed rate × Principal / 12
  - Principal adjustment: Principal × CPI change

- **Projection:** Use current principal value, adjust monthly with CPI
- **Currency:** ILS (convert to USD for unified view)

**Loan Payment Cash Flow:**

```python
@dataclass
class LoanPaymentCashFlow:
    """Cash flow from loan payment."""
    payment_date: datetime
    payment_amount: float  # Negative for outflows
    currency: str  # ILS or USD
    loan_type: str  # "SHIR" or "CPI_LINKED"
    principal_remaining: float
    interest_payment: float
    principal_payment: float
    shir_rate: Optional[float] = None  # For SHIR-based loans
    cpi_change: Optional[float] = None  # For CPI-linked loans
```

### 2. Option Expiration Cash Flows

**Long Options (Cash Inflows at Expiration):**

- **Intrinsic Value:** max(0, Spot - Strike) for calls, max(0, Strike - Spot) for puts
- **Cash Flow:** Intrinsic value × Contract multiplier × Quantity
- **For Box Spreads:** Cash flow = Strike width × Contract multiplier × Quantity (guaranteed)

**Short Options (Cash Outflows at Expiration if ITM):**

- **Intrinsic Value:** -max(0, Spot - Strike) for calls, -max(0, Strike - Spot) for puts
- **Cash Flow:** Intrinsic value × Contract multiplier × Quantity (negative if ITM)

**Option Expiration Cash Flow:**

```python
@dataclass
class OptionExpirationCashFlow:
    """Cash flow from option expiration."""
    expiration_date: datetime
    option_type: str  # "call" or "put"
    position_type: str  # "long" or "short"
    symbol: str
    strike: float
    quantity: int  # Number of contracts
    contract_multiplier: int  # Typically 100
    projected_intrinsic_value: float  # Based on current spot
    projected_cash_flow: float  # Intrinsic value × multiplier × quantity
    underlying_price: float  # Current underlying price for projection
    is_box_spread: bool = False  # Box spreads have guaranteed cash flow
    box_strike_width: Optional[float] = None  # For box spreads
```

**Box Spread Cash Flow Calculation:**

```python
def calculate_box_spread_cash_flow(
    spread: BoxSpreadLeg,
    quantity: int
) -> OptionExpirationCashFlow:
    """Calculate guaranteed cash flow from box spread at expiration."""
    strike_width = spread.get_strike_width()  # K2 - K1
    contract_multiplier = 100

    # Box spreads guarantee strike width payout at expiration
    guaranteed_cash_flow = strike_width * contract_multiplier * quantity

    return OptionExpirationCashFlow(
        expiration_date=parse_expiry(spread.long_call.expiry),
        option_type="box_spread",
        position_type="long",  # Assuming long box spread
        symbol=spread.long_call.symbol,
        strike=0.0,  # N/A for box spread
        quantity=quantity,
        contract_multiplier=contract_multiplier,
        projected_intrinsic_value=strike_width,
        projected_cash_flow=guaranteed_cash_flow,
        underlying_price=0.0,  # N/A
        is_box_spread=True,
        box_strike_width=strike_width
    )
```

### 3. Bond Cash Flows

**Bond Coupon Payments (Periodic Inflows):**

- **Payment Frequency:** Semi-annual (most bonds), quarterly, or annual
- **Payment Amount:** Face value × Coupon rate / Payment frequency
- **Example:** $10,000 bond with 5% annual coupon = $250 semi-annually

**Bond Principal at Maturity (Large Inflow):**

- **Payment Date:** Bond maturity date
- **Payment Amount:** Face value (principal returned)
- **Example:** $10,000 bond matures → $10,000 cash inflow

**Bond ETF Cash Flows:**

- **Distribution Frequency:** Monthly or quarterly
- **Distribution Amount:** Based on underlying bond portfolio
- **Yield:** Annual yield divided by distribution frequency

**Bond Cash Flow:**

```python
@dataclass
class BondCashFlow:
    """Cash flow from bond coupon or maturity."""
    payment_date: datetime
    cash_flow_type: str  # "coupon" or "maturity"
    bond_symbol: str
    face_value: float
    coupon_rate: float
    payment_amount: float  # Positive for inflows
    currency: str  # USD, ILS, etc.
    is_bond_etf: bool = False  # True for bond ETFs (distributions instead of coupons)
    distribution_yield: Optional[float] = None  # For bond ETFs
```

**Bond Coupon Calculation:**

```python
def calculate_bond_coupon_schedule(
    bond: Position,
    coupon_rate: float,
    payment_frequency: int,  # 2 for semi-annual, 4 for quarterly, 12 for monthly
    maturity_date: datetime
) -> List[BondCashFlow]:
    """Calculate all coupon payments until maturity."""
    cash_flows = []
    face_value = bond.quantity  # Assuming quantity is face value
    coupon_payment = face_value * (coupon_rate / payment_frequency)

    # Calculate payment dates
    current_date = datetime.now()
    payment_date = current_date

    # Adjust to next payment date
    # For semi-annual: next payment in 0 or 6 months
    # For quarterly: next payment in 0, 3, 6, or 9 months
    # (Simplified - actual calculation would use bond issue date and payment calendar)

    while payment_date < maturity_date:
        cash_flows.append(BondCashFlow(
            payment_date=payment_date,
            cash_flow_type="coupon",
            bond_symbol=bond.symbol,
            face_value=face_value,
            coupon_rate=coupon_rate,
            payment_amount=coupon_payment,
            currency=bond.currency
        ))

        # Advance to next payment date
        months_to_add = 12 / payment_frequency
        payment_date = add_months(payment_date, months_to_add)

    # Add maturity principal payment
    cash_flows.append(BondCashFlow(
        payment_date=maturity_date,
        cash_flow_type="maturity",
        bond_symbol=bond.symbol,
        face_value=face_value,
        coupon_rate=coupon_rate,
        payment_amount=face_value,  # Principal returned
        currency=bond.currency
    ))

    return cash_flows
```

### 4. Dividend Cash Flows

**Stock/ETF Dividends (Periodic Inflows):**

- **Payment Frequency:** Quarterly (most common), monthly, semi-annual, or annual
- **Payment Amount:** Dividend per share × Number of shares
- **Ex-Dividend Date:** Shares must be held before this date to receive dividend
- **Payment Date:** Dividend payment date (typically 1-2 months after ex-date)

**Dividend Cash Flow:**

```python
@dataclass
class DividendCashFlow:
    """Cash flow from dividend payment."""
    ex_dividend_date: datetime  # Date shares must be held by
    payment_date: datetime  # Actual dividend payment date
    symbol: str
    dividend_per_share: float
    quantity: float  # Number of shares
    total_dividend: float  # dividend_per_share × quantity
    currency: str  # USD, ILS, etc.
    frequency: str  # "quarterly", "monthly", "semi-annual", "annual"
    dividend_yield: Optional[float] = None  # Annual dividend yield
```

**Dividend Schedule Retrieval:**

- Use ORATS API (existing integration): `ORATSClient.get_dividend_schedule()`
- Use IBKR API: Contract details include dividend information
- For TASE securities: Use TASE dividend announcements

## Cash Flow Timeline Generation

### Aggregate Cash Flow Timeline

```python
@dataclass
class CashFlowTimeline:
    """Aggregated cash flow timeline for portfolio."""
    cash_flows: List[CashFlowEvent]  # Sorted by date
    total_inflows: float  # Sum of positive cash flows
    total_outflows: float  # Sum of negative cash flows
    net_cash_flow: float  # Total inflows - total outflows
    cumulative_balance: Dict[datetime, float]  # Projected cash balance over time
    currency: str  # Base currency (USD)

    def get_cash_flow_by_date_range(
        self,
        start_date: datetime,
        end_date: datetime
    ) -> List[CashFlowEvent]:
        """Get cash flows within date range."""
        return [
            cf for cf in self.cash_flows
            if start_date <= cf.date <= end_date
        ]

    def get_net_cash_flow_by_period(
        self,
        period_days: int = 30
    ) -> Dict[datetime, float]:
        """Get net cash flow aggregated by period (e.g., monthly)."""
        # Group cash flows by period and sum
        pass

    def project_cash_balance(
        self,
        current_cash: float,
        start_date: datetime,
        end_date: datetime
    ) -> Dict[datetime, float]:
        """Project cash balance over time given current cash and future flows."""
        balance = current_cash
        projected = {start_date: balance}

        for cf in self.cash_flows:
            if start_date <= cf.date <= end_date:
                balance += cf.amount
                projected[cf.date] = balance

        return projected

@dataclass
class CashFlowEvent:
    """Single cash flow event."""
    date: datetime
    amount: float  # Positive for inflows, negative for outflows
    currency: str
    cash_flow_type: str  # "loan_payment", "option_expiration", "bond_coupon", "bond_maturity", "dividend"
    description: str
    position_id: Optional[str] = None  # Link to position if applicable
    loan_id: Optional[str] = None  # Link to loan if applicable
```

## Implementation

### Cash Flow Calculator

```python

# python/integration/cash_flow_calculator.py

from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import List, Dict, Optional
from enum import Enum

class CashFlowType(Enum):
    LOAN_PAYMENT = "loan_payment"
    OPTION_EXPIRATION = "option_expiration"
    BOND_COUPON = "bond_coupon"
    BOND_MATURITY = "bond_maturity"
    DIVIDEND = "dividend"

@dataclass
class CashFlowEvent:
    """Single cash flow event."""
    date: datetime
    amount: float  # Positive for inflows, negative for outflows
    currency: str  # USD, ILS, etc.
    cash_flow_type: CashFlowType
    description: str
    position_id: Optional[str] = None
    loan_id: Optional[str] = None
    # Type-specific fields
    underlying_price: Optional[float] = None  # For options
    strike: Optional[float] = None  # For options
    coupon_rate: Optional[float] = None  # For bonds
    dividend_per_share: Optional[float] = None  # For dividends

class CashFlowCalculator:
    """Calculate future cash flows from all portfolio positions and loans."""

    def __init__(
        self,
        orats_client=None,  # For dividend schedules
        ibkr_client=None,  # For option/bond details
        fx_rates: Dict[str, float] = None  # ILS/USD, etc.
    ):
        self.orats_client = orats_client
        self.ibkr_client = ibkr_client
        self.fx_rates = fx_rates or {}

    def calculate_portfolio_cash_flows(
        self,
        positions: List[Position],
        loans: List[Loan],
        forecast_horizon_days: int = 365
    ) -> CashFlowTimeline:
        """
        Calculate all future cash flows for portfolio.

        Args:
            positions: List of all positions (IBKR + Israeli brokers)
            loans: List of loans (SHIR-based + CPI-linked)
            forecast_horizon_days: Number of days to forecast ahead

        Returns:
            CashFlowTimeline with all cash flows sorted by date
        """
        all_cash_flows = []
        end_date = datetime.now() + timedelta(days=forecast_horizon_days)

        # Calculate loan payment cash flows
        for loan in loans:
            loan_cash_flows = self.calculate_loan_cash_flows(loan, end_date)
            all_cash_flows.extend(loan_cash_flows)

        # Calculate option expiration cash flows
        for position in positions:
            if position.instrument_type == "option":
                option_cash_flows = self.calculate_option_cash_flows(position, end_date)
                all_cash_flows.extend(option_cash_flows)

        # Calculate bond cash flows (coupons + maturity)
        for position in positions:
            if position.instrument_type == "bond" or position.instrument_type == "bond_etf":
                bond_cash_flows = self.calculate_bond_cash_flows(position, end_date)
                all_cash_flows.extend(bond_cash_flows)

        # Calculate dividend cash flows
        for position in positions:
            if position.instrument_type in ["stock", "etf"]:
                dividend_cash_flows = self.calculate_dividend_cash_flows(position, end_date)
                all_cash_flows.extend(dividend_cash_flows)

        # Sort by date
        all_cash_flows.sort(key=lambda cf: cf.date)

        # Convert all to base currency (USD)
        for cf in all_cash_flows:
            if cf.currency != "USD":
                fx_rate = self.fx_rates.get(f"{cf.currency}_USD", 1.0)
                cf.amount *= fx_rate
                cf.currency = "USD"

        # Calculate totals and cumulative balance
        total_inflows = sum(cf.amount for cf in all_cash_flows if cf.amount > 0)
        total_outflows = sum(abs(cf.amount) for cf in all_cash_flows if cf.amount < 0)
        net_cash_flow = total_inflows - total_outflows

        # Generate cumulative balance projection
        cumulative_balance = self._calculate_cumulative_balance(all_cash_flows)

        return CashFlowTimeline(
            cash_flows=all_cash_flows,
            total_inflows=total_inflows,
            total_outflows=total_outflows,
            net_cash_flow=net_cash_flow,
            cumulative_balance=cumulative_balance,
            currency="USD"
        )

    def calculate_loan_cash_flows(
        self,
        loan: Loan,
        end_date: datetime
    ) -> List[CashFlowEvent]:
        """Calculate loan payment cash flows."""
        cash_flows = []
        current_date = datetime.now()

        while current_date <= end_date:
            # Calculate next payment date (monthly)
            next_payment_date = add_months(current_date, 1)
            if next_payment_date > end_date:
                break

            # Calculate payment amount
            if loan.loan_type == "SHIR":
                # Variable SHIR-based loan
                payment_amount = self._calculate_shir_loan_payment(loan)
            else:  # CPI_LINKED
                # Fixed rate CPI-linked loan
                payment_amount = self._calculate_cpi_linked_loan_payment(loan)

            cash_flows.append(CashFlowEvent(
                date=next_payment_date,
                amount=-payment_amount,  # Negative for outflows
                currency=loan.currency,
                cash_flow_type=CashFlowType.LOAN_PAYMENT,
                description=f"{loan.loan_type} loan payment",
                loan_id=loan.id
            ))

            current_date = next_payment_date

        return cash_flows

    def calculate_option_cash_flows(
        self,
        position: Position,
        end_date: datetime
    ) -> List[CashFlowEvent]:
        """Calculate option expiration cash flows."""
        if not position.expiration_date:
            return []

        if position.expiration_date > end_date:
            return []

        # For box spreads, calculate guaranteed cash flow
        if position.is_box_spread():
            strike_width = position.box_strike_width
            contract_multiplier = 100
            guaranteed_cash_flow = strike_width * contract_multiplier * position.quantity

            return [CashFlowEvent(
                date=position.expiration_date,
                amount=guaranteed_cash_flow,  # Positive for long box spread
                currency=position.currency,
                cash_flow_type=CashFlowType.OPTION_EXPIRATION,
                description=f"Box spread expiration: {position.symbol}",
                position_id=position.id,
                underlying_price=position.current_price,
                strike=position.strike
            )]

        # For regular options, project intrinsic value
        intrinsic_value = self._calculate_option_intrinsic_value(
            position,
            position.current_price  # Use current price as projection
        )
        contract_multiplier = 100
        cash_flow_amount = intrinsic_value * contract_multiplier * position.quantity

        # For short options, cash flow is negative if ITM
        if position.position_type == "short":
            cash_flow_amount = -abs(cash_flow_amount)

        return [CashFlowEvent(
            date=position.expiration_date,
            amount=cash_flow_amount,
            currency=position.currency,
            cash_flow_type=CashFlowType.OPTION_EXPIRATION,
            description=f"Option expiration: {position.symbol} {position.option_type} {position.strike}",
            position_id=position.id,
            underlying_price=position.current_price,
            strike=position.strike
        )]

    def calculate_bond_cash_flows(
        self,
        position: Position,
        end_date: datetime
    ) -> List[CashFlowEvent]:
        """Calculate bond coupon and maturity cash flows."""
        cash_flows = []

        # Get bond details (coupon rate, maturity date, payment frequency)
        bond_details = self._get_bond_details(position)
        if not bond_details:
            return []

        # Calculate coupon payments
        if bond_details.maturity_date > end_date:
            # Calculate all coupon payments until end_date
            coupon_payments = self._calculate_coupon_schedule(
                position,
                bond_details.coupon_rate,
                bond_details.payment_frequency,
                end_date
            )
            cash_flows.extend(coupon_payments)
        else:
            # Bond matures before end_date
            # Add remaining coupon payments + maturity
            coupon_payments = self._calculate_coupon_schedule(
                position,
                bond_details.coupon_rate,
                bond_details.payment_frequency,
                bond_details.maturity_date
            )
            cash_flows.extend(coupon_payments)

            # Add maturity principal payment
            cash_flows.append(CashFlowEvent(
                date=bond_details.maturity_date,
                amount=position.quantity,  # Face value returned
                currency=position.currency,
                cash_flow_type=CashFlowType.BOND_MATURITY,
                description=f"Bond maturity: {position.symbol}",
                position_id=position.id,
                coupon_rate=bond_details.coupon_rate
            ))

        return cash_flows

    def calculate_dividend_cash_flows(
        self,
        position: Position,
        end_date: datetime
    ) -> List[CashFlowEvent]:
        """Calculate dividend cash flows."""
        cash_flows = []

        # Get dividend schedule from ORATS or IBKR
        dividend_schedule = self._get_dividend_schedule(position.symbol)
        if not dividend_schedule:
            return []

        # Project dividends based on historical frequency
        current_date = datetime.now()
        dividend_per_share = dividend_schedule.get("next_div_amount", 0.0)
        frequency = dividend_schedule.get("div_frequency", "quarterly")  # quarterly, monthly, etc.

        if dividend_per_share == 0.0:
            return []

        # Calculate payment frequency in months
        months_per_payment = {
            "quarterly": 3,
            "monthly": 1,
            "semi-annual": 6,
            "annual": 12
        }.get(frequency, 3)

        next_payment_date = parse_date(dividend_schedule.get("next_div_ex_date"))
        if not next_payment_date:
            # Estimate next payment based on frequency
            next_payment_date = current_date + timedelta(days=30 * months_per_payment)

        while next_payment_date <= end_date:
            total_dividend = dividend_per_share * position.quantity

            cash_flows.append(CashFlowEvent(
                date=next_payment_date,
                amount=total_dividend,  # Positive for inflows
                currency=position.currency,
                cash_flow_type=CashFlowType.DIVIDEND,
                description=f"Dividend payment: {position.symbol}",
                position_id=position.id,
                dividend_per_share=dividend_per_share
            ))

            # Advance to next payment date
            next_payment_date = add_months(next_payment_date, months_per_payment)

        return cash_flows

    def _calculate_shir_loan_payment(self, loan: Loan) -> float:
        """Calculate SHIR-based loan payment."""
        # Get current SHIR rate
        shir_rate = self._get_current_shir_rate()
        effective_rate = shir_rate + loan.spread

        # Monthly payment = Principal × (effective_rate / 12)
        # Simplified - actual calculation would use amortization formula
        monthly_interest = loan.principal_remaining * (effective_rate / 12)
        # Add principal payment (simplified - actual would use amortization schedule)
        monthly_principal = loan.principal_remaining / loan.remaining_months

        return monthly_interest + monthly_principal

    def _calculate_cpi_linked_loan_payment(self, loan: Loan) -> float:
        """Calculate CPI-linked loan payment."""
        # Fixed interest payment
        monthly_interest = loan.principal_remaining * (loan.fixed_rate / 12)

        # Principal adjustment for CPI (simplified - actual would track CPI changes)
        # For projection, assume current CPI growth rate continues
        cpi_growth_rate = self._get_current_cpi_growth_rate()  # Annual rate
        monthly_cpi_adjustment = loan.principal_remaining * (cpi_growth_rate / 12)

        # Monthly principal payment (simplified)
        monthly_principal = loan.principal_remaining / loan.remaining_months

        # CPI-linked loans: principal increases with CPI, but payment adjusts
        # (Simplified - actual calculation more complex)
        return monthly_interest + monthly_principal + monthly_cpi_adjustment

    def _calculate_option_intrinsic_value(
        self,
        option: Position,
        underlying_price: float
    ) -> float:
        """Calculate option intrinsic value."""
        if option.option_type == "call":
            return max(0.0, underlying_price - option.strike)
        else:  # put
            return max(0.0, option.strike - underlying_price)

    def _calculate_coupon_schedule(
        self,
        bond: Position,
        coupon_rate: float,
        payment_frequency: int,
        end_date: datetime
    ) -> List[CashFlowEvent]:
        """Calculate bond coupon payment schedule."""
        cash_flows = []
        face_value = bond.quantity  # Assuming quantity is face value
        coupon_payment = face_value * (coupon_rate / payment_frequency)

        # Get next payment date (simplified - actual would use bond issue date)
        current_date = datetime.now()
        months_per_payment = 12 / payment_frequency

        # Estimate next payment date (simplified)
        next_payment_date = current_date + timedelta(days=30 * months_per_payment)

        while next_payment_date <= end_date:
            cash_flows.append(CashFlowEvent(
                date=next_payment_date,
                amount=coupon_payment,
                currency=bond.currency,
                cash_flow_type=CashFlowType.BOND_COUPON,
                description=f"Bond coupon: {bond.symbol}",
                position_id=bond.id,
                coupon_rate=coupon_rate
            ))

            next_payment_date = add_months(next_payment_date, months_per_payment)

        return cash_flows

    def _get_dividend_schedule(self, symbol: str) -> Optional[Dict]:
        """Get dividend schedule from ORATS or IBKR."""
        if self.orats_client:
            return self.orats_client.get_dividend_schedule(symbol)
        # Fallback to IBKR or other sources
        return None

    def _get_bond_details(self, position: Position) -> Optional[BondDetails]:
        """Get bond details (coupon rate, maturity date, payment frequency)."""
        # For bond ETFs, use ETF prospectus data
        if position.symbol == "TLT":
            return BondDetails(
                coupon_rate=0.04,  # Approximate
                maturity_date=None,  # ETFs don't mature
                payment_frequency=12,  # Monthly distributions
                is_etf=True
            )
        elif position.symbol == "SHY":
            return BondDetails(
                coupon_rate=0.03,  # Approximate
                maturity_date=None,
                payment_frequency=12,
                is_etf=True
            )

        # For individual bonds, get from IBKR contract details
        if self.ibkr_client:
            contract_details = self.ibkr_client.get_contract_details(position.symbol)
            if contract_details:
                return BondDetails(
                    coupon_rate=contract_details.get("coupon_rate", 0.0),
                    maturity_date=parse_date(contract_details.get("maturity_date")),
                    payment_frequency=contract_details.get("payment_frequency", 2),
                    is_etf=False
                )

        return None

    def _calculate_cumulative_balance(
        self,
        cash_flows: List[CashFlowEvent],
        initial_cash: float = 0.0
    ) -> Dict[datetime, float]:
        """Calculate cumulative cash balance over time."""
        balance = initial_cash
        cumulative = {}

        for cf in cash_flows:
            balance += cf.amount
            cumulative[cf.date] = balance

        return cumulative

    def _get_current_shir_rate(self) -> float:
        """Get current SHIR rate (would fetch from Bank of Israel or IBKR)."""
        # TODO: Implement SHIR rate fetching
        return 0.045  # Placeholder: 4.5%

    def _get_current_cpi_growth_rate(self) -> float:
        """Get current CPI growth rate (would fetch from Bank of Israel)."""
        # TODO: Implement CPI growth rate fetching
        return 0.03  # Placeholder: 3% annual CPI growth
```

## Cash Flow Integration with Investment Strategy

### Cash Flow-Aware Cash Management

**Updated Tier 1: Immediate Cash (3-5% of portfolio)**

- **Components:**
  1. **Emergency Reserve:** 2-3% for unexpected needs
  2. **Loan Payment Reserve:** Reserve for upcoming loan payments (next 2 months)
  3. **Margin Buffer:** Reserve for margin requirements
  4. **Cash Flow Buffer:** Reserve for upcoming cash flow timing mismatches

**Cash Flow-Based Liquidity Planning:**

```python
def calculate_minimum_cash_reserve(
    portfolio_value: float,
    cash_flow_timeline: CashFlowTimeline,
    loan_payments: List[Loan],
    days_ahead: int = 60
) -> float:
    """
    Calculate minimum cash reserve needed based on upcoming cash flows.

    Args:
        portfolio_value: Total portfolio value
        cash_flow_timeline: Future cash flow projections
        loan_payments: List of loans
        days_ahead: Days to look ahead for cash flow planning

    Returns:
        Minimum cash reserve needed
    """
    end_date = datetime.now() + timedelta(days=days_ahead)

    # Calculate maximum cumulative outflow in next N days
    upcoming_flows = cash_flow_timeline.get_cash_flow_by_date_range(
        datetime.now(),
        end_date
    )

    # Calculate cumulative balance assuming zero starting cash
    cumulative_balance = cash_flow_timeline.project_cash_balance(
        current_cash=0.0,
        start_date=datetime.now(),
        end_date=end_date
    )

    # Find minimum (most negative) balance
    min_balance = min(cumulative_balance.values()) if cumulative_balance else 0.0

    # Reserve minimum balance + buffer (20% safety margin)
    minimum_reserve = abs(min_balance) * 1.20 if min_balance < 0 else 0.0

    # Add emergency reserve (2% of portfolio)
    emergency_reserve = portfolio_value * 0.02

    return minimum_reserve + emergency_reserve
```

### Cash Flow-Based Rebalancing Triggers

Add to rebalancing triggers:

- Upcoming large cash outflow (loan payment, option expiration) exceeds available cash
- Upcoming large cash inflow (bond maturity, option expiration) changes allocation targets
- Cumulative cash balance projected to go negative in next 30 days
- Cash flow timing mismatch (inflows don't cover outflows in time window)

### Spare Cash Allocation with Cash Flow Awareness

**Updated Spare Cash Allocation Algorithm:**

```python
def calculate_spare_cash_allocation_with_cash_flow(
    total_spare_cash: float,
    cash_flow_timeline: CashFlowTimeline,
    current_tbill_rate: float,
    best_box_spread_rate: float,
    short_bond_rate: float,
    ibkr_cash_rate: float,
    days_to_next_opportunity: int,
    liquidity_needs: float
) -> SpareCashAllocation:
    """
    Calculate spare cash allocation considering upcoming cash flows.

    Allocates more to liquid assets if large cash outflows are upcoming.
    Allocates more to illiquid assets if large cash inflows are upcoming.
    """
    # Check upcoming cash flows in next 30 days
    next_30_days = datetime.now() + timedelta(days=30)
    upcoming_flows = cash_flow_timeline.get_cash_flow_by_date_range(
        datetime.now(),
        next_30_days
    )

    # Calculate net cash flow in next 30 days
    net_cash_flow_30d = sum(cf.amount for cf in upcoming_flows)

    # If large outflow upcoming, favor liquidity
    if net_cash_flow_30d < -total_spare_cash * 0.30:  # Large outflow (>30% of spare cash)
        # Favor IBKR cash and T-bills for liquidity
        return {
            .box_spread_percent = 0.10,  # Minimal (illiquid)
            .tbill_percent = 0.50,       # High (liquid)
            .short_bond_percent = 0.10,
            .ibkr_cash_percent = 0.30,   # High (most liquid)
            .immediate_cash_percent = 0.10
        }

    # If large inflow upcoming, can allocate more to illiquid assets
    if net_cash_flow_30d > total_spare_cash * 0.30:  # Large inflow
        # Can favor box spreads (illiquid but higher yield)
        return {
            .box_spread_percent = 0.50,  # High (illiquid but high yield)
            .tbill_percent = 0.20,
            .short_bond_percent = 0.10,
            .ibkr_cash_percent = 0.10,
            .immediate_cash_percent = 0.10
        }

    # Normal allocation (use existing algorithm)
    return calculate_allocation(
        total_spare_cash,
        current_tbill_rate,
        best_box_spread_rate,
        short_bond_rate,
        ibkr_cash_rate,
        margin_interest_rate,
        margin_requirement,
        excess_liquidity,
        days_to_next_opportunity,
        liquidity_needs
    )
```

## Data Models

### Loan Data Model

```python
@dataclass
class Loan:
    """Loan liability model."""
    id: str
    loan_type: str  # "SHIR" or "CPI_LINKED"
    currency: str  # ILS
    principal_remaining: float
    monthly_payment: float  # Current monthly payment
    fixed_rate: Optional[float] = None  # For CPI-linked loans
    spread: Optional[float] = None  # For SHIR-based loans (spread over SHIR)
    remaining_months: int
    next_payment_date: datetime
    shir_rate: Optional[float] = None  # Current SHIR rate (for SHIR loans)
    cpi_index: Optional[float] = None  # Current CPI index (for CPI-linked loans)
```

## Integration with Backend

### Backend API Extensions

**Add to SystemSnapshot:**

```rust
// agents/backend/crates/api/src/state.rs

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct SystemSnapshot {
    // ... existing fields ...
    pub cash_flow_timeline: CashFlowTimeline,
}

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct CashFlowTimeline {
    pub cash_flows: Vec<CashFlowEvent>,
    pub total_inflows: f64,
    pub total_outflows: f64,
    pub net_cash_flow: f64,
    pub cumulative_balance: HashMap<String, f64>,  // date -> balance
    pub forecast_horizon_days: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct CashFlowEvent {
    pub date: String,  // ISO 8601 date
    pub amount: f64,   // Positive for inflows, negative for outflows
    pub currency: String,
    pub cash_flow_type: String,  // "loan_payment", "option_expiration", "bond_coupon", "bond_maturity", "dividend"
    pub description: String,
    pub position_id: Option<String>,
    pub loan_id: Option<String>,
}
```

**New Backend Endpoint:**

```rust
// agents/backend/crates/api/src/rest.rs
.route("/api/v1/cash-flow/timeline", get(cash_flow_timeline))
.route("/api/v1/cash-flow/projection", get(cash_flow_projection))

async fn cash_flow_timeline(
    State(state): State<RestState>,
    Query(params): Query<CashFlowQuery>
) -> Json<CashFlowTimeline> {
    // Calculate and return cash flow timeline
    let timeline = calculate_cash_flow_timeline(&state, params.days_ahead);
    Json(timeline)
}

async fn cash_flow_projection(
    State(state): State<RestState>,
    Query(params): Query<CashFlowProjectionQuery>
) -> Json<CashFlowProjection> {
    // Project cash balance over time
    let projection = project_cash_balance(&state, params.days_ahead);
    Json(projection)
}
```

## Cash Flow Alerts

**Alert Conditions:**

- Upcoming cash outflow exceeds available cash in next 7 days
- Cumulative cash balance projected to go negative in next 30 days
- Large cash inflow upcoming (bond maturity, option expiration) - opportunity alert
- Loan payment due within 3 days - reminder alert
- Option expiration approaching within 7 days - decision alert (close vs. let expire)

## Implementation Roadmap

### Phase 1: Cash Flow Calculation Framework (Week 1-2)

- [ ] Design cash flow data models
- [ ] Implement CashFlowCalculator class
- [ ] Implement loan payment cash flow calculation
- [ ] Implement option expiration cash flow calculation
- [ ] Implement bond coupon/maturity cash flow calculation
- [ ] Implement dividend cash flow calculation
- [ ] Create cash flow timeline aggregation

### Phase 2: Integration with Investment Strategy (Week 3)

- [ ] Integrate cash flow calculator with PortfolioAllocationManager
- [ ] Update cash management strategy with cash flow awareness
- [ ] Add cash flow-based liquidity planning
- [ ] Update spare cash allocation algorithm
- [ ] Add cash flow-based rebalancing triggers

### Phase 3: Backend Integration (Week 4)

- [ ] Add cash flow timeline to SystemSnapshot
- [ ] Create backend API endpoints for cash flow data
- [ ] Integrate with existing snapshot endpoint
- [ ] Add cash flow alerts
- [ ] Test end-to-end cash flow forecasting

### Phase 4: Data Source Integration (Week 5)

- [ ] Integrate ORATS dividend schedules
- [ ] Integrate IBKR contract details (bonds, options)
- [ ] Integrate SHIR rate fetching (Bank of Israel or IBKR)
- [ ] Integrate CPI data (Bank of Israel)
- [ ] Add TASE bond coupon/maturity data

## Testing

### Unit Tests

- Loan payment calculation (SHIR-based, CPI-linked)
- Option expiration cash flow calculation
- Bond coupon schedule calculation
- Dividend cash flow projection
- Cash flow timeline aggregation
- Currency conversion

### Integration Tests

- End-to-end cash flow timeline generation
- Cash flow-based liquidity planning
- Cash flow-aware allocation calculation
- Backend API cash flow endpoints

## References

- [Net Present Value (NPV)](https://en.wikipedia.org/wiki/Net_present_value)
- Bond Duration and Convexity
- Option Expiration Settlement
- Dividend Payment Schedules

---

**Next Steps:**

1. Review and approve design
2. Begin Phase 1 implementation (cash flow calculation framework)
3. Integrate with existing investment strategy framework
