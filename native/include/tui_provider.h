// tui_provider.h - Data provider interface for TUI
#pragma once

#include "tui_data.h"
#include <memory>
#include <thread>
#include <atomic>
#include <queue>
#include <mutex>
#include <condition_variable>

namespace tui {

// Provider produces snapshots for the TUI.
class Provider {
public:
  virtual ~Provider() = default;

  // Start the provider (begin emitting snapshots)
  virtual void Start() = 0;

  // Stop the provider
  virtual void Stop() = 0;

  // Get the latest snapshot (non-blocking, returns empty snapshot if none available)
  virtual Snapshot GetSnapshot() = 0;

  // Check if provider is running
  virtual bool IsRunning() const = 0;
};

// MockProvider generates synthetic snapshots for local testing.
class MockProvider : public Provider {
public:
  MockProvider();
  ~MockProvider() override;

  void Start() override;
  void Stop() override;
  Snapshot GetSnapshot() override;
  bool IsRunning() const override;

  // Add a symbol to the mock provider's rotation
  void AddSymbol(const std::string& symbol);

private:
  void GenerateLoop();
  Snapshot GenerateSnapshot();

  std::atomic<bool> running_{false};
  std::thread worker_;
  std::mutex mutex_;
  Snapshot latest_snapshot_;
  std::vector<std::string> symbols_;
  std::chrono::milliseconds interval_{1000};

  // Simple PRNG state (mutable so it can be modified in const methods)
  mutable uint64_t rand_state_ = 1;
  double RandFloat() const;
  double BasePriceForSymbol(const std::string& symbol) const;
  SymbolSnapshot MockSymbol(const std::string& symbol, double base,
                          std::chrono::system_clock::time_point now);
  Position MockPosition(const std::string& name, int qty,
                       std::chrono::system_clock::time_point now);
  Candle MockCandle(double current, double base,
                   std::chrono::system_clock::time_point now);
  std::vector<OptionSeries> MockOptionChains(double last,
                                             std::chrono::system_clock::time_point now);
  std::vector<HistoryEntry> MockHistory(std::chrono::system_clock::time_point now);
  std::vector<YieldCurvePoint> MockYieldCurve(std::chrono::system_clock::time_point now);
  std::vector<FAQEntry> MockFAQs();
};

// RestProvider polls a REST endpoint for snapshots.
class RestProvider : public Provider {
public:
  RestProvider(const std::string& endpoint, std::chrono::milliseconds interval);
  ~RestProvider() override;

  void Start() override;
  void Stop() override;
  Snapshot GetSnapshot() override;
  bool IsRunning() const override;

private:
  void PollLoop();
  Snapshot Fetch();

  std::string endpoint_;
  std::chrono::milliseconds interval_;
  std::atomic<bool> running_{false};
  std::thread worker_;
  std::mutex mutex_;
  Snapshot latest_snapshot_;
};

// IBKRRestProvider uses IBKR Client Portal REST API for real-time data.
// Requires Client Portal Gateway running locally (https://localhost:5000)
class IBKRRestProvider : public Provider {
public:
  IBKRRestProvider(const std::string& base_url,
                   const std::string& account_id,
                   bool verify_ssl,
                   std::chrono::milliseconds interval);
  ~IBKRRestProvider() override;

  void Start() override;
  void Stop() override;
  Snapshot GetSnapshot() override;
  bool IsRunning() const override;

private:
  void PollLoop();
  Snapshot FetchFromIBKR();
  bool EnsureSession();
  std::vector<std::string> GetAccounts();

  std::string base_url_;
  std::string account_id_;
  bool verify_ssl_;
  std::chrono::milliseconds interval_;
  std::atomic<bool> running_{false};
  std::thread worker_;
  std::mutex mutex_;
  Snapshot latest_snapshot_;
  std::string active_account_id_;  // Cached account ID
};

// LiveVolProvider uses Cboe LiveVol API for options market data and analytics.
// See: https://api.livevol.com/v1/docs/
// Requires OAuth 2.0 authentication (client_id, client_secret)
class LiveVolProvider : public Provider {
public:
  LiveVolProvider(const std::string& base_url,
                  const std::string& client_id,
                  const std::string& client_secret,
                  bool use_real_time,
                  std::chrono::milliseconds interval);
  ~LiveVolProvider() override;

  void Start() override;
  void Stop() override;
  Snapshot GetSnapshot() override;
  bool IsRunning() const override;

private:
  void PollLoop();
  Snapshot FetchFromLiveVol();
  bool EnsureAccessToken();
  std::string GetAccessToken();

  std::string base_url_;
  std::string client_id_;
  std::string client_secret_;
  bool use_real_time_;
  std::chrono::milliseconds interval_;
  std::atomic<bool> running_{false};
  std::thread worker_;
  std::mutex mutex_;
  Snapshot latest_snapshot_;
  std::string access_token_;  // Cached OAuth token
  std::chrono::system_clock::time_point token_expiry_;
};

// FileProvider polls a JSON file on disk for snapshots (broker-agnostic).
class FileProvider : public Provider {
public:
  FileProvider(const std::string& file_path, std::chrono::milliseconds interval);
  ~FileProvider() override;

  void Start() override;
  void Stop() override;
  Snapshot GetSnapshot() override;
  bool IsRunning() const override;

private:
  void PollLoop();
  Snapshot LoadFromFile();

  std::string file_path_;
  std::chrono::milliseconds interval_;
  std::atomic<bool> running_{false};
  std::thread worker_;
  std::mutex mutex_;
  Snapshot latest_snapshot_;
  std::time_t last_mtime_{0};
};

} // namespace tui
