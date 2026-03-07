// test_asset_relationship.cpp - Tests for AssetRelationshipGraph (Phase 2)
#include "asset_relationship.h"
#include "collateral_valuator.h"
#include "financing_instrument.h"

#include <catch2/catch_test_macros.hpp>

using namespace synthetic;

TEST_CASE("AssetRelationshipGraph add and get_collateral_for", "[synthetic][asset_relationship]") {
  AssetRelationshipGraph graph;

  AssetRelationship rel;
  rel.source_asset_id = "T-BILL-3M-USD";
  rel.target_asset_id = "SPX-OPTIONS";
  rel.type = RelationshipType::COLLATERAL;
  rel.collateral_value_ratio = 0.95;
  rel.base_currency = "USD";
  rel.target_currency = "USD";
  rel.is_active = true;
  rel.valid_from = std::chrono::system_clock::now();
  rel.valid_until = {};

  graph.add_relationship(rel);

  auto collateral = graph.get_collateral_for("SPX-OPTIONS", "USD");
  REQUIRE(collateral.size() == 1u);
  REQUIRE(collateral[0].source_asset_id == "T-BILL-3M-USD");
  REQUIRE(collateral[0].target_asset_id == "SPX-OPTIONS");
  REQUIRE(collateral[0].collateral_value_ratio == 0.95);
}

TEST_CASE("AssetRelationshipGraph get_financing_options filters by currency and amount",
          "[synthetic][asset_relationship]") {
  AssetRelationshipGraph graph;

  AssetRelationship rel;
  rel.source_asset_id = "BOX-SPREAD-SPX";
  rel.target_asset_id = "USD";
  rel.type = RelationshipType::FINANCING;
  rel.base_currency = "USD";
  rel.min_amount = 10000.0;
  rel.max_amount = 500000.0;
  rel.min_days_to_maturity = 7;
  rel.max_days_to_maturity = 365;
  rel.is_active = true;
  rel.valid_from = std::chrono::system_clock::now();

  graph.add_relationship(rel);

  auto options = graph.get_financing_options("USD", 50000.0, 30);
  REQUIRE(options.size() == 1u);
  REQUIRE(options[0].source_asset_id == "BOX-SPREAD-SPX");

  options = graph.get_financing_options("USD", 1000.0, 30);
  REQUIRE(options.empty());

  options = graph.get_financing_options("USD", 50000.0, 3);
  REQUIRE(options.empty());
}

TEST_CASE("AssetRelationshipGraph get_cross_currency_paths", "[synthetic][asset_relationship]") {
  AssetRelationshipGraph graph;

  AssetRelationship rel;
  rel.source_asset_id = "FX-SWAP-USD-ILS";
  rel.target_asset_id = "ILS";
  rel.type = RelationshipType::CROSS_CURRENCY;
  rel.base_currency = "USD";
  rel.target_currency = "ILS";
  rel.is_active = true;
  rel.valid_from = std::chrono::system_clock::now();

  graph.add_relationship(rel);

  auto paths = graph.get_cross_currency_paths("USD", "ILS");
  REQUIRE(paths.size() == 1u);
  REQUIRE(paths[0].source_asset_id == "FX-SWAP-USD-ILS");
}

TEST_CASE("CollateralValuator value_for_margin applies haircut", "[synthetic][collateral]") {
  CollateralValuator valuator;
  CollateralPosition pos;
  pos.asset_id = "T-BILL-3M";
  pos.market_value = 100000.0;
  pos.currency = "USD";
  pos.days_to_maturity = 60;

  auto v = valuator.value_for_margin(pos, "OPTIONS", "IBKR");
  REQUIRE(v.gross_value == 100000.0);
  REQUIRE(v.net_collateral_value <= v.gross_value);
  REQUIRE(v.margin_credit == v.net_collateral_value);
  REQUIRE(v.currency == "USD");
}

TEST_CASE("FinancingInstrumentRegistry register and get_best_financing", "[synthetic][registry]") {
  FinancingInstrumentRegistry registry;

  FinancingInstrument inst;
  inst.type = InstrumentType::BOX_SPREAD;
  inst.asset_id = "BOX-SPX-30";
  inst.symbol = "SPX";
  inst.base_currency = "USD";
  inst.annual_rate = 4.5;
  inst.effective_rate = 4.6;
  inst.days_to_maturity = 30;
  inst.min_size = 10000.0;
  inst.max_size = 1000000.0;
  inst.liquidity_score = 80.0;
  inst.is_available = true;
  registry.register_instrument(inst);

  FinancingInstrument inst2;
  inst2.type = InstrumentType::BANK_LOAN;
  inst2.asset_id = "BANK-USD";
  inst2.base_currency = "USD";
  inst2.annual_rate = 5.0;
  inst2.effective_rate = 5.2;
  inst2.min_size = 0.0;
  inst2.max_size = 500000.0;
  inst2.liquidity_score = 90.0;
  inst2.is_available = true;
  registry.register_instrument(inst2);

  auto best = registry.get_best_financing("USD", 50000.0, 30, "COST");
  REQUIRE(best.asset_id == "BOX-SPX-30");
  REQUIRE(best.effective_rate < 5.0);
}
