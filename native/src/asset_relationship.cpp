// asset_relationship.cpp - Asset relationship graph implementation (Phase 2)
#include "asset_relationship.h"

#include <algorithm>
#include <chrono>

namespace synthetic {

void AssetRelationshipGraph::add_relationship(const AssetRelationship& rel) {
  relationships_.push_back(rel);
}

bool AssetRelationshipGraph::is_relationship_valid_at(
    const AssetRelationship& rel,
    std::chrono::system_clock::time_point at) {
  if (!rel.is_active) return false;
  if (at < rel.valid_from) return false;
  if (rel.valid_until != std::chrono::system_clock::time_point{} && at > rel.valid_until)
    return false;
  return true;
}

std::vector<AssetRelationship> AssetRelationshipGraph::get_collateral_for(
    const std::string& target_asset_id,
    const Currency& currency) const {
  const auto now = std::chrono::system_clock::now();
  std::vector<AssetRelationship> out;
  for (const auto& rel : relationships_) {
    if (rel.type != RelationshipType::COLLATERAL && rel.type != RelationshipType::MARGIN)
      continue;
    if (rel.target_asset_id != target_asset_id) continue;
    if (!rel.target_currency.empty() && rel.target_currency != currency) continue;
    if (!is_relationship_valid_at(rel, now)) continue;
    out.push_back(rel);
  }
  return out;
}

std::vector<AssetRelationship> AssetRelationshipGraph::get_financing_options(
    const Currency& currency,
    double amount,
    int days_needed) const {
  const auto now = std::chrono::system_clock::now();
  std::vector<AssetRelationship> out;
  for (const auto& rel : relationships_) {
    if (rel.type != RelationshipType::FINANCING) continue;
    if (!rel.base_currency.empty() && rel.base_currency != currency) continue;
    if (rel.min_amount > 0.0 && amount < rel.min_amount) continue;
    if (rel.max_amount > 0.0 && amount > rel.max_amount) continue;
    if (rel.min_days_to_maturity > 0 && days_needed < rel.min_days_to_maturity) continue;
    if (rel.max_days_to_maturity > 0 && days_needed > rel.max_days_to_maturity) continue;
    if (!is_relationship_valid_at(rel, now)) continue;
    out.push_back(rel);
  }
  return out;
}

std::vector<AssetRelationship> AssetRelationshipGraph::find_collateral_chain(
    const std::string& target_asset_id,
    double required_margin,
    const Currency& currency) const {
  (void)required_margin;  // Phase 2: simple chain; optimization in later phase
  return get_collateral_for(target_asset_id, currency);
}

std::vector<AssetRelationship> AssetRelationshipGraph::get_cross_currency_paths(
    const Currency& from,
    const Currency& to) const {
  const auto now = std::chrono::system_clock::now();
  std::vector<AssetRelationship> out;
  for (const auto& rel : relationships_) {
    if (rel.type != RelationshipType::CROSS_CURRENCY) continue;
    if (rel.base_currency != from || rel.target_currency != to) continue;
    if (!is_relationship_valid_at(rel, now)) continue;
    out.push_back(rel);
  }
  return out;
}

}  // namespace synthetic
