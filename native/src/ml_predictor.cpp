// ml_predictor.cpp - XGBoost ML model integration implementation
#include "ib_box_spread/ml_predictor.h"

#include <algorithm>
#include <chrono>
#include <cmath>
#include <fstream>
#include <sstream>

// XGBoost C API
// Note: This requires XGBoost to be built and linked
// For now, we provide a stub implementation that can be completed when XGBoost
// is integrated
#ifdef ENABLE_XGBOOST
#include <xgboost/c_api.h>
#else
// Stub definitions for when XGBoost is not available
typedef void *BoosterHandle;
typedef void *DMatrixHandle;
#endif

namespace ml {

// ============================================================================
// XGBoost Model Wrapper
// ============================================================================

struct XGBoostModel {
#ifdef ENABLE_XGBOOST
  BoosterHandle booster;
  bool is_loaded;

  XGBoostModel() : booster(nullptr), is_loaded(false) {}

  ~XGBoostModel() {
    if (booster) {
      XGBoosterFree(booster);
    }
  }
#else
  bool is_loaded;
  std::string model_path; // Store path for future loading

  XGBoostModel() : is_loaded(false) {}
  ~XGBoostModel() = default;
#endif
};

// ============================================================================
// ML Predictor Implementation
// ============================================================================

MLPredictor::MLPredictor()
    : profitability_model_(std::make_unique<XGBoostModel>()),
      risk_model_(std::make_unique<XGBoostModel>()),
      execution_timing_model_(std::make_unique<XGBoostModel>()),
      position_sizing_model_(std::make_unique<XGBoostModel>()) {}

MLPredictor::~MLPredictor() = default;

bool MLPredictor::load_models(const std::string &models_dir) {
  bool all_loaded = true;

  // Load feature names first
  if (!load_feature_names(models_dir)) {
    // Feature names are optional, but log warning
  }

  // Load all models
  all_loaded &=
      load_profitability_model(models_dir + "/profitability_model.bin");
  all_loaded &= load_risk_model(models_dir + "/risk_model.bin");
  all_loaded &=
      load_execution_timing_model(models_dir + "/execution_timing_model.bin");
  all_loaded &=
      load_position_sizing_model(models_dir + "/position_sizing_model.bin");

  return all_loaded;
}

bool MLPredictor::load_profitability_model(const std::string &model_path) {
#ifdef ENABLE_XGBOOST
  if (XGBoosterCreate(nullptr, 0, &profitability_model_->booster) != 0) {
    return false;
  }

  if (XGBoosterLoadModel(profitability_model_->booster, model_path.c_str()) !=
      0) {
    XGBoosterFree(profitability_model_->booster);
    profitability_model_->booster = nullptr;
    return false;
  }

  profitability_model_->is_loaded = true;
  return true;
#else
  // Stub: store path for future loading
  profitability_model_->model_path = model_path;
  profitability_model_->is_loaded =
      false; // Not actually loaded without XGBoost
  return false;
#endif
}

bool MLPredictor::load_risk_model(const std::string &model_path) {
#ifdef ENABLE_XGBOOST
  if (XGBoosterCreate(nullptr, 0, &risk_model_->booster) != 0) {
    return false;
  }

  if (XGBoosterLoadModel(risk_model_->booster, model_path.c_str()) != 0) {
    XGBoosterFree(risk_model_->booster);
    risk_model_->booster = nullptr;
    return false;
  }

  risk_model_->is_loaded = true;
  return true;
#else
  risk_model_->model_path = model_path;
  risk_model_->is_loaded = false;
  return false;
#endif
}

bool MLPredictor::load_execution_timing_model(const std::string &model_path) {
#ifdef ENABLE_XGBOOST
  if (XGBoosterCreate(nullptr, 0, &execution_timing_model_->booster) != 0) {
    return false;
  }

  if (XGBoosterLoadModel(execution_timing_model_->booster,
                         model_path.c_str()) != 0) {
    XGBoosterFree(execution_timing_model_->booster);
    execution_timing_model_->booster = nullptr;
    return false;
  }

  execution_timing_model_->is_loaded = true;
  return true;
#else
  execution_timing_model_->model_path = model_path;
  execution_timing_model_->is_loaded = false;
  return false;
#endif
}

bool MLPredictor::load_position_sizing_model(const std::string &model_path) {
#ifdef ENABLE_XGBOOST
  if (XGBoosterCreate(nullptr, 0, &position_sizing_model_->booster) != 0) {
    return false;
  }

  if (XGBoosterLoadModel(position_sizing_model_->booster, model_path.c_str()) !=
      0) {
    XGBoosterFree(position_sizing_model_->booster);
    position_sizing_model_->booster = nullptr;
    return false;
  }

  position_sizing_model_->is_loaded = true;
  return true;
#else
  position_sizing_model_->model_path = model_path;
  position_sizing_model_->is_loaded = false;
  return false;
#endif
}

bool MLPredictor::is_profitability_model_loaded() const {
  return profitability_model_->is_loaded;
}

bool MLPredictor::is_risk_model_loaded() const {
  return risk_model_->is_loaded;
}

bool MLPredictor::is_execution_timing_model_loaded() const {
  return execution_timing_model_->is_loaded;
}

bool MLPredictor::is_position_sizing_model_loaded() const {
  return position_sizing_model_->is_loaded;
}

std::vector<float>
MLPredictor::extract_features(const types::BoxSpreadLeg &spread,
                              const option_chain::OptionChain &chain,
                              double underlying_price, double risk_free_rate,
                              double vix, double market_volatility) const {
  std::vector<float> features;

  // Extract all feature groups
  auto spread_features = extract_spread_features(spread);
  auto leg_features = extract_leg_features(spread, chain);
  auto market_features = extract_market_features(
      spread, underlying_price, risk_free_rate, vix, market_volatility);
  auto greeks_features = extract_greeks_features(spread);
  auto temporal_features = extract_temporal_features(spread);

  // Combine all features
  features.insert(features.end(), spread_features.begin(),
                  spread_features.end());
  features.insert(features.end(), leg_features.begin(), leg_features.end());
  features.insert(features.end(), market_features.begin(),
                  market_features.end());
  features.insert(features.end(), greeks_features.begin(),
                  greeks_features.end());
  features.insert(features.end(), temporal_features.begin(),
                  temporal_features.end());

  return features;
}

std::vector<float>
MLPredictor::extract_spread_features(const types::BoxSpreadLeg &spread) const {
  std::vector<float> features;

  // Box value = (max_strike - min_strike) * multiplier
  double min_strike =
      std::min({spread.long_call.strike, spread.short_call.strike,
                spread.long_put.strike, spread.short_put.strike});
  double max_strike =
      std::max({spread.long_call.strike, spread.short_call.strike,
                spread.long_put.strike, spread.short_put.strike});
  double box_value = max_strike - min_strike;

  // Net premium (simplified calculation)
  double net_premium = 0.0; // Would need actual bid/ask from chain

  // Spread width (total bid-ask across all legs)
  double total_spread = 0.0; // Would need actual spreads from chain

  features.push_back(static_cast<float>(box_value));
  features.push_back(static_cast<float>(net_premium));
  features.push_back(static_cast<float>(total_spread));

  return features;
}

std::vector<float> MLPredictor::extract_leg_features(
    const types::BoxSpreadLeg &spread,
    const option_chain::OptionChain &chain) const {
  std::vector<float> features;

  // Volume and open interest statistics
  // This would need to query the chain for actual values
  features.push_back(0.0f); // total_volume
  features.push_back(0.0f); // avg_volume
  features.push_back(0.0f); // volume_std
  features.push_back(0.0f); // min_volume
  features.push_back(0.0f); // max_volume
  features.push_back(0.0f); // total_oi
  features.push_back(0.0f); // avg_oi
  features.push_back(0.0f); // oi_std
  features.push_back(0.0f); // min_oi
  features.push_back(0.0f); // max_oi

  return features;
}

std::vector<float> MLPredictor::extract_market_features(
    const types::BoxSpreadLeg &spread, double underlying_price,
    double risk_free_rate, double vix, double market_volatility) const {
  std::vector<float> features;

  // Moneyness
  double avg_strike = (spread.long_call.strike + spread.short_call.strike +
                       spread.long_put.strike + spread.short_put.strike) /
                      4.0;
  double moneyness = (underlying_price - avg_strike) / underlying_price;

  features.push_back(static_cast<float>(underlying_price));
  features.push_back(static_cast<float>(risk_free_rate));
  features.push_back(static_cast<float>(moneyness));
  features.push_back(static_cast<float>(vix));
  features.push_back(static_cast<float>(market_volatility));

  return features;
}

std::vector<float>
MLPredictor::extract_greeks_features(const types::BoxSpreadLeg &spread) const {
  std::vector<float> features;

  // Net Greeks (would need actual Greeks from chain)
  features.push_back(0.0f); // net_delta
  features.push_back(0.0f); // net_gamma
  features.push_back(0.0f); // net_theta
  features.push_back(0.0f); // net_vega

  return features;
}

std::vector<float> MLPredictor::extract_temporal_features(
    const types::BoxSpreadLeg &spread) const {
  std::vector<float> features;

  auto now = std::chrono::system_clock::now();
  auto time_t = std::chrono::system_clock::to_time_t(now);
  std::tm *time_info = std::localtime(&time_t);

  // Time to expiration (simplified - would need actual expiration dates)
  double days_to_exp = 30.0; // Placeholder

  features.push_back(static_cast<float>(days_to_exp));
  features.push_back(static_cast<float>(time_info->tm_hour));
  features.push_back(static_cast<float>(time_info->tm_min));
  features.push_back(
      (time_info->tm_hour >= 9 && time_info->tm_hour < 16) ? 1.0f : 0.0f);

  return features;
}

std::vector<float>
MLPredictor::predict_with_model(XGBoostModel *model,
                                const std::vector<float> &features) const {
#ifdef ENABLE_XGBOOST
  if (!model || !model->is_loaded) {
    return {};
  }

  // Create DMatrix from features
  DMatrixHandle dmat;
  XGDMatrixCreateFromMat(features.data(),
                         1,                                       // nrow
                         static_cast<bst_ulong>(features.size()), // ncol
                         -1, // missing value
                         &dmat);

  // Predict
  bst_ulong out_len;
  const float *out_result;
  XGBoosterPredict(model->booster, dmat,
                   0, // option_mask
                   0, // ntree_limit
                   &out_len, &out_result);

  std::vector<float> predictions(out_result, out_result + out_len);

  XGDMatrixFree(dmat);

  return predictions;
#else
  // Stub: return empty vector
  (void)model;
  (void)features;
  return {};
#endif
}

std::optional<ProfitabilityPrediction> MLPredictor::predict_profitability(
    const types::BoxSpreadLeg &spread, const option_chain::OptionChain &chain,
    double underlying_price, double risk_free_rate, double vix,
    double market_volatility, double threshold) const {
  if (!is_profitability_model_loaded()) {
    return std::nullopt;
  }

  auto features = extract_features(spread, chain, underlying_price,
                                   risk_free_rate, vix, market_volatility);

  auto predictions = predict_with_model(profitability_model_.get(), features);

  if (predictions.empty()) {
    return std::nullopt;
  }

  double probability = static_cast<double>(predictions[0]);

  ProfitabilityPrediction result;
  result.probability = probability;
  result.is_profitable = probability >= threshold;
  result.confidence = std::abs(probability - 0.5) * 2.0; // Distance from 0.5

  return result;
}

std::optional<RiskPrediction> MLPredictor::predict_risk(
    const types::BoxSpreadLeg &spread, const option_chain::OptionChain &chain,
    double underlying_price, double risk_free_rate, double vix,
    double market_volatility, double threshold) const {
  if (!is_risk_model_loaded()) {
    return std::nullopt;
  }

  auto features = extract_features(spread, chain, underlying_price,
                                   risk_free_rate, vix, market_volatility);

  auto predictions = predict_with_model(risk_model_.get(), features);

  if (predictions.empty()) {
    return std::nullopt;
  }

  double risk_score = static_cast<double>(predictions[0]);

  RiskPrediction result;
  result.risk_score = risk_score;
  result.is_high_risk = risk_score >= threshold;
  result.risk_factors =
      "ML model prediction"; // Could be enhanced with SHAP values

  return result;
}

std::optional<ExecutionTimingPrediction> MLPredictor::predict_execution_timing(
    const types::BoxSpreadLeg &spread, const option_chain::OptionChain &chain,
    double underlying_price, double risk_free_rate, double vix,
    double market_volatility) const {
  if (!is_execution_timing_model_loaded()) {
    return std::nullopt;
  }

  auto features = extract_features(spread, chain, underlying_price,
                                   risk_free_rate, vix, market_volatility);

  auto predictions =
      predict_with_model(execution_timing_model_.get(), features);

  if (predictions.empty()) {
    return std::nullopt;
  }

  double wait_seconds = static_cast<double>(predictions[0]);

  ExecutionTimingPrediction result;
  result.optimal_wait_seconds = std::max(0.0, wait_seconds);
  result.urgency_score =
      std::min(1.0, 1.0 / (1.0 + wait_seconds /
                                     60.0)); // Higher urgency for shorter waits
  result.execute_now = wait_seconds < 5.0;   // Execute now if wait < 5 seconds

  return result;
}

std::optional<PositionSizingPrediction> MLPredictor::predict_position_size(
    const types::BoxSpreadLeg &spread, const option_chain::OptionChain &chain,
    double underlying_price, double risk_free_rate, double account_equity,
    double current_portfolio_risk, double vix, double market_volatility) const {
  if (!is_position_sizing_model_loaded()) {
    return std::nullopt;
  }

  // Add account-specific features
  auto features = extract_features(spread, chain, underlying_price,
                                   risk_free_rate, vix, market_volatility);

  // Append account features
  features.push_back(static_cast<float>(account_equity));
  features.push_back(static_cast<float>(current_portfolio_risk));

  auto predictions = predict_with_model(position_sizing_model_.get(), features);

  if (predictions.empty()) {
    return std::nullopt;
  }

  double normalized_size =
      std::clamp(static_cast<double>(predictions[0]), 0.0, 1.0);

  PositionSizingPrediction result;
  result.normalized_size = normalized_size;
  result.recommended_size =
      normalized_size * account_equity / 10000.0; // Simplified calculation
  result.confidence = 0.8;                        // Placeholder

  return result;
}

MLPredictor::CombinedPrediction MLPredictor::predict_all(
    const types::BoxSpreadLeg &spread, const option_chain::OptionChain &chain,
    double underlying_price, double risk_free_rate, double account_equity,
    double current_portfolio_risk, double vix, double market_volatility) const {
  CombinedPrediction result;

  // Extract features once
  auto features = extract_features(spread, chain, underlying_price,
                                   risk_free_rate, vix, market_volatility);

  // Make all predictions
  result.profitability = predict_profitability(
      spread, chain, underlying_price, risk_free_rate, vix, market_volatility);

  result.risk = predict_risk(spread, chain, underlying_price, risk_free_rate,
                             vix, market_volatility);

  result.execution_timing = predict_execution_timing(
      spread, chain, underlying_price, risk_free_rate, vix, market_volatility);

  result.position_sizing = predict_position_size(
      spread, chain, underlying_price, risk_free_rate, account_equity,
      current_portfolio_risk, vix, market_volatility);

  return result;
}

std::vector<std::string> MLPredictor::get_feature_names() const {
  return feature_names_;
}

std::string MLPredictor::get_model_info() const {
  std::ostringstream oss;
  oss << "ML Predictor Models:\n";
  oss << "  Profitability: "
      << (is_profitability_model_loaded() ? "Loaded" : "Not loaded") << "\n";
  oss << "  Risk: " << (is_risk_model_loaded() ? "Loaded" : "Not loaded")
      << "\n";
  oss << "  Execution Timing: "
      << (is_execution_timing_model_loaded() ? "Loaded" : "Not loaded") << "\n";
  oss << "  Position Sizing: "
      << (is_position_sizing_model_loaded() ? "Loaded" : "Not loaded") << "\n";
  oss << "  Features: " << feature_names_.size() << "\n";
  return oss.str();
}

bool MLPredictor::load_feature_names(const std::string &models_dir) {
  std::ifstream file(models_dir + "/feature_names.json");
  if (!file.is_open()) {
    return false;
  }

  // Simple JSON parsing (could use nlohmann/json if available)
  // For now, just try to read as array
  std::string content((std::istreambuf_iterator<char>(file)),
                      std::istreambuf_iterator<char>());

  // Basic parsing - would need proper JSON parser
  // This is a placeholder
  feature_names_.clear();

  return true;
}

} // namespace ml
