from python.integration.frontend_views import (
    infer_relationships,
    normalize_bank_accounts_to_positions,
    relationship_nodes,
)


def test_normalize_bank_accounts_expands_mixed_currency_accounts() -> None:
    positions = normalize_bank_accounts_to_positions(
        [
            {
                "account_name": "Discount Main",
                "is_mixed_currency": True,
                "balances_by_currency": {"ILS": 1200.0, "USD": -50.0},
                "credit_rate": 0.03,
            }
        ]
    )

    assert [position["name"] for position in positions] == [
        "Discount Main (ILS)",
        "Discount Main (USD)",
    ]
    assert [position["currency"] for position in positions] == ["ILS", "USD"]
    assert positions[0]["roi"] == 3.0


def test_infer_relationships_uses_bank_accounts_with_debit_rate_as_loans() -> None:
    positions = [
        {
            "name": "SPX Box",
            "instrument_type": "box_spread",
            "cash_flow": 1000.0,
            "candle": {"close": 1000.0},
        },
        {
            "name": "Treasury ETF",
            "instrument_type": "bond",
            "rate": 0.09,
            "cash_flow": 800.0,
            "candle": {"close": 800.0},
        },
    ]
    bank_accounts = [
        {
            "account_name": "Broker Credit Line",
            "balance": -500.0,
            "debit_rate": 0.08,
        }
    ]

    relationships = infer_relationships(positions, bank_accounts)

    assert any(
        rel["from"] == "Broker Credit Line"
        and rel["to"] == "SPX Box"
        and rel["type"] == "margin"
        for rel in relationships
    )
    assert any(
        rel["from"] == "Treasury ETF"
        and rel["to"] == "Broker Credit Line"
        and rel["type"] == "collateral"
        for rel in relationships
    )


def test_relationship_nodes_include_positions_and_bank_accounts() -> None:
    relationships = [
        {"from": "Loan A", "to": "Box A"},
        {"from": "Box A", "to": "Synthetic Financing"},
    ]
    positions = [{"name": "Bond A"}]
    bank_accounts = [{"account_name": "Checking"}]

    nodes = relationship_nodes(relationships, positions, bank_accounts)

    assert nodes == ["Bond A", "Box A", "Checking", "Loan A", "Synthetic Financing"]
