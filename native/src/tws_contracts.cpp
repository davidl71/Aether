// tws_contracts.cpp - Contract details lookups and conId resolution
#include "tws_contracts.h"
#include "tws_connection.h"
#include "tws_conversions.h"
#include "connection_utils.h"
#include <spdlog/spdlog.h>

#include "EClientSocket.h"
#include "Contract.h"

#include <thread>

namespace tws {

ContractHandler::ContractHandler(EClientSocket& client)
    : client_(client) {}

// ============================================================================
// Public interface
// ============================================================================

int ContractHandler::request_contract_details(
    const types::OptionContract& contract,
    ContractDetailsCallback callback,
    ConnectionHandler& conn)
{
  if (conn.is_mock_mode())
  {
    int request_id = conn.claim_request_id();
    long mock_conId = 1000000 + request_id;
    {
      std::lock_guard<std::mutex> lock(mutex_);
      callbacks_[request_id] = callback;
      results_[request_id] = mock_conId;
    }
    std::thread([this, request_id]() {
      std::this_thread::sleep_for(std::chrono::milliseconds(50));
      ContractDetailsCallback cb;
      long conId = -1;
      {
        std::lock_guard<std::mutex> lock(mutex_);
        auto it = callbacks_.find(request_id);
        if (it == callbacks_.end()) return;
        cb = it->second;
        conId = results_[request_id];
        callbacks_.erase(request_id);
        results_.erase(request_id);
      }
      if (cb) cb(conId);
    }).detach();
    return request_id;
  }

  if (!conn.is_connected())
  {
    spdlog::error("Cannot request contract details: Not connected");
    return -1;
  }

  if (!conn.check_rate_limit())
  {
    spdlog::error("Rate limit exceeded for contract details: {}", contract.to_string());
    return -1;
  }

  int request_id = conn.claim_request_id();
  Contract tws_contract = convert_to_tws_contract(contract);

  {
    std::lock_guard<std::mutex> lock(mutex_);
    callbacks_[request_id] = callback;
  }

  conn.record_rate_message();
  client_.reqContractDetails(request_id, tws_contract);

  spdlog::debug("Requested contract details for {} (id={})",
                contract.to_string(), request_id);
  return request_id;
}

long ContractHandler::request_contract_details_sync(
    const types::OptionContract& contract, int timeout_ms,
    ConnectionHandler& conn)
{
  if (conn.is_mock_mode())
  {
    int request_id = conn.claim_request_id();
    return 1000000 + request_id;
  }

  if (!conn.is_connected())
  {
    spdlog::error("Cannot request contract details: Not connected");
    return -1;
  }

  if (!conn.check_rate_limit())
  {
    spdlog::error("Rate limit exceeded for sync contract details: {}",
                  contract.to_string());
    return -1;
  }

  int request_id = conn.claim_request_id();

  auto promise = std::make_shared<std::promise<long>>();
  auto future = promise->get_future();

  {
    std::lock_guard<std::mutex> lock(mutex_);
    promises_[request_id] = promise;
  }

  Contract tws_contract = convert_to_tws_contract(contract);
  conn.record_rate_message();
  client_.reqContractDetails(request_id, tws_contract);

  if (future.wait_for(std::chrono::milliseconds(timeout_ms)) == std::future_status::timeout)
  {
    spdlog::warn("Contract details timeout for {} (id={}, timeout={}ms)",
                 contract.to_string(), request_id, timeout_ms);
    std::lock_guard<std::mutex> lock(mutex_);
    promises_.erase(request_id);
    return -1;
  }

  long conId = future.get();
  {
    std::lock_guard<std::mutex> lock(mutex_);
    promises_.erase(request_id);
  }
  return conId;
}

// ============================================================================
// EWrapper callback forwarding
// ============================================================================

void ContractHandler::on_contract_details(int req_id, long con_id)
{
  try
  {
    spdlog::debug("Contract details received: reqId={}, conId={}", req_id, con_id);

    ContractDetailsCallback callback;
    std::shared_ptr<std::promise<long>> promise;

    {
      std::lock_guard<std::mutex> lock(mutex_);
      results_[req_id] = con_id;

      if (callbacks_.count(req_id))
      {
        callback = callbacks_[req_id];
        callbacks_.erase(req_id);
      }
      if (promises_.count(req_id))
      {
        promise = promises_[req_id];
        promises_.erase(req_id);
      }
    }

    if (callback) callback(con_id);
    if (promise) promise->set_value(con_id);
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in contractDetails(reqId={}): {}", req_id, e.what());
  }
}

void ContractHandler::on_contract_details_end(int req_id)
{
  try
  {
    spdlog::debug("Contract details end for reqId={}", req_id);

    std::lock_guard<std::mutex> lock(mutex_);
    if (promises_.count(req_id))
    {
      if (!results_.count(req_id))
      {
        spdlog::warn("Contract details end without result for reqId={}", req_id);
        promises_[req_id]->set_value(-1);
      }
      promises_.erase(req_id);
    }
    if (callbacks_.count(req_id))
    {
      spdlog::warn("Contract details end without result for async reqId={}", req_id);
      callbacks_.erase(req_id);
    }
    results_.erase(req_id);
  }
  catch (const std::exception& e)
  {
    spdlog::error("Exception in contractDetailsEnd(reqId={}): {}", req_id, e.what());
  }
}

} // namespace tws
