#pragma once

#include "tws_client.h"

#include <future>
#include <map>
#include <mutex>
#include <string>

class EClientSocket;

namespace tws {

class ConnectionHandler;

class ContractHandler
{
public:
  explicit ContractHandler(EClientSocket& client);

  int request_contract_details(const types::OptionContract& contract,
                               ContractDetailsCallback callback,
                               ConnectionHandler& conn);
  long request_contract_details_sync(const types::OptionContract& contract,
                                     int timeout_ms, ConnectionHandler& conn);

  // EWrapper callback forwarding
  void on_contract_details(int req_id, long con_id);
  void on_contract_details_end(int req_id);

private:
  EClientSocket& client_;
  mutable std::mutex mutex_;
  std::map<int, ContractDetailsCallback> callbacks_;
  std::map<int, long> results_;
  std::map<int, std::shared_ptr<std::promise<long>>> promises_;
};

} // namespace tws
