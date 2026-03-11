use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IbPositionDto {
    pub account_id: Option<String>,
    pub symbol: String,
    pub name: String,
    pub conid: Option<i64>,
    pub quantity: f64,
    pub avg_price: f64,
    pub current_price: Option<f64>,
    pub market_value: Option<f64>,
    pub unrealized_pl: Option<f64>,
}

impl IbPositionDto {
    pub fn from_portal_position(raw: &Value, account_id: Option<&str>) -> Option<Self> {
        let Value::Object(map) = raw else {
            return None;
        };

        let conid = value_as_i64(map.get("conid"));
        let asset_class = value_as_str(map.get("assetClass")).unwrap_or_default();
        let maturity_date = value_as_str(map.get("maturityDate"))
            .or_else(|| value_as_str(map.get("maturity_date")));
        let raw_name = value_as_str(map.get("ticker"))
            .or_else(|| value_as_str(map.get("symbol")))
            .or_else(|| value_as_str(map.get("contractDesc")))
            .unwrap_or_else(|| conid.map(|value| value.to_string()).unwrap_or_default());
        let name = format_ibcid_display_name(&raw_name, &asset_class, conid, maturity_date.as_deref());
        let symbol = if name.trim().is_empty() {
            raw_name
        } else {
            name.clone()
        };

        Some(Self {
            account_id: account_id.map(str::to_string),
            symbol,
            name,
            conid,
            quantity: value_as_f64(map.get("position")).unwrap_or_default(),
            avg_price: value_as_f64(map.get("avgCost"))
                .or_else(|| value_as_f64(map.get("averageCost")))
                .unwrap_or_default(),
            current_price: value_as_f64(map.get("markPrice"))
                .or_else(|| value_as_f64(map.get("lastPrice"))),
            market_value: value_as_f64(map.get("mktValue"))
                .or_else(|| value_as_f64(map.get("markValue"))),
            unrealized_pl: value_as_f64(map.get("unrealizedPnl")),
        })
    }
}

fn value_as_str(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) => {
            let trimmed = text.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        }
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn value_as_i64(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn value_as_f64(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn format_ibcid_display_name(
    raw_name: &str,
    asset_class: &str,
    conid: Option<i64>,
    maturity_date: Option<&str>,
) -> String {
    if raw_name.trim().is_empty() || conid.is_none() {
        return raw_name.trim().to_string();
    }

    let conid = conid.expect("checked above");
    let trimmed = raw_name.trim();
    let is_ibcid = trimmed.to_ascii_uppercase().starts_with("IBCID")
        || (trimmed.chars().all(|ch| ch.is_ascii_digit()) && trimmed == conid.to_string());
    if !is_ibcid {
        return trimmed.to_string();
    }

    let label = match asset_class.trim().to_ascii_uppercase().as_str() {
        "BILL" | "TBILL" => "T-Bill",
        "BOND" => "Bond",
        _ => return trimmed.to_string(),
    };

    let maturity_prefix = maturity_date
        .and_then(|date| (date.len() >= 10).then(|| &date[..10]))
        .map(|date| format!("{date} "))
        .unwrap_or_default();

    format!("{label} {maturity_prefix}({conid})").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::IbPositionDto;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize)]
    struct DiscountBankCompatPosition {
        symbol: String,
        quantity: f64,
        avg_price: f64,
        current_price: Option<f64>,
        market_value: Option<f64>,
        unrealized_pl: Option<f64>,
    }

    #[test]
    fn normalizes_standard_equity_position() {
        let raw = json!({
            "ticker": "SPY",
            "conid": "756733",
            "position": "100",
            "averageCost": "450.0",
            "markPrice": "451.0",
            "markValue": "45100.0",
            "unrealizedPnl": "100.0"
        });

        let dto = IbPositionDto::from_portal_position(&raw, Some("DU123")).expect("dto");

        assert_eq!(dto.account_id.as_deref(), Some("DU123"));
        assert_eq!(dto.symbol, "SPY");
        assert_eq!(dto.name, "SPY");
        assert_eq!(dto.conid, Some(756733));
        assert_eq!(dto.quantity, 100.0);
        assert_eq!(dto.avg_price, 450.0);
        assert_eq!(dto.current_price, Some(451.0));
        assert_eq!(dto.market_value, Some(45100.0));
        assert_eq!(dto.unrealized_pl, Some(100.0));
    }

    #[test]
    fn normalizes_bond_like_ibcid_name() {
        let raw = json!({
            "symbol": "123456789",
            "assetClass": "BOND",
            "conid": 123456789,
            "maturityDate": "2030-01-15",
            "position": 5,
            "avgCost": 98.25
        });

        let dto = IbPositionDto::from_portal_position(&raw, None).expect("dto");

        assert_eq!(dto.symbol, "Bond 2030-01-15 (123456789)");
        assert_eq!(dto.name, "Bond 2030-01-15 (123456789)");
        assert_eq!(dto.account_id, None);
    }

    #[test]
    fn tolerates_missing_optional_market_fields() {
        let raw = json!({
            "ticker": "QQQ",
            "position": "10",
            "averageCost": "500.0"
        });

        let dto = IbPositionDto::from_portal_position(&raw, Some("DU999")).expect("dto");

        assert_eq!(dto.current_price, None);
        assert_eq!(dto.market_value, None);
        assert_eq!(dto.unrealized_pl, None);
    }

    #[test]
    fn invalid_numeric_fields_fall_back_predictably() {
        let raw = json!({
            "ticker": "IWM",
            "position": "not-a-number",
            "avgCost": "bad",
            "markPrice": "oops"
        });

        let dto = IbPositionDto::from_portal_position(&raw, None).expect("dto");

        assert_eq!(dto.quantity, 0.0);
        assert_eq!(dto.avg_price, 0.0);
        assert_eq!(dto.current_price, None);
    }

    #[test]
    fn projects_to_discount_bank_compatibility_shape() {
        let dto = IbPositionDto {
            account_id: Some("DU123".to_string()),
            symbol: "SPY".to_string(),
            name: "SPY".to_string(),
            conid: Some(756733),
            quantity: 100.0,
            avg_price: 450.0,
            current_price: Some(451.0),
            market_value: Some(45100.0),
            unrealized_pl: Some(100.0),
        };

        let projected = serde_json::to_value(&dto).expect("serialize");
        let compat: DiscountBankCompatPosition =
            serde_json::from_value(projected).expect("compat deserialize");

        assert_eq!(compat.symbol, "SPY");
        assert_eq!(compat.quantity, 100.0);
        assert_eq!(compat.avg_price, 450.0);
        assert_eq!(compat.current_price, Some(451.0));
        assert_eq!(compat.market_value, Some(45100.0));
        assert_eq!(compat.unrealized_pl, Some(100.0));
    }
}
