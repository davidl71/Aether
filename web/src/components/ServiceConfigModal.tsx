import { useState, useEffect, useCallback } from 'react';
import type { BackendServiceStatus } from '../hooks/useBackendServices';

interface ServiceConfigModalProps {
  service: BackendServiceStatus;
  onClose: () => void;
  onRefresh: () => void;
}

export function ServiceConfigModal({ service, onClose, onRefresh }: ServiceConfigModalProps) {
  const [logs, setLogs] = useState<string[]>([]);
  const [loadingLogs, setLoadingLogs] = useState(false);
  const [logError, setLogError] = useState<string | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [config, setConfig] = useState({
    port: service.port,
    healthCheckInterval: 10000,
    timeout: 2000,
  });

  // Map service names to log file names
  const getLogFileName = (serviceName: string): string => {
    const logMap: Record<string, string> = {
      'Alpaca': 'alpaca-service.log',
      'TradeStation': 'tradestation-service.log',
      'IB': 'ib-service.log',
      'Discount Bank': 'discount-bank-service.log',
      'Risk-Free Rate': 'risk-free-rate-service.log',
      'Tastytrade': 'tastytrade-service.log',
      'Rust Backend': 'rust-backend.log', // May not exist, but placeholder
    };
    return logMap[serviceName] || `${serviceName.toLowerCase().replace(/\s+/g, '-')}-service.log`;
  };

  const fetchLogs = useCallback(() => {
    setLoadingLogs(true);
    setLogError(null);

    try {
      // In a real implementation, you'd have a backend endpoint to fetch logs
      // For now, we'll use a mock API endpoint or show console logs
      // Since we can't directly read files from the browser, we'll:
      // 1. Show recent console logs related to this service
      // 2. Provide instructions for viewing logs

      // For now, we'll create a placeholder that shows service info
      // In production, you'd call: `/api/services/${service.name}/logs`
      const logFileName = getLogFileName(service.name);

      // Simulate log fetching (replace with actual API call)
      const mockLogs = [
        `[${new Date().toISOString()}] Service: ${service.name}`,
        `[${new Date().toISOString()}] Port: ${service.port}`,
        `[${new Date().toISOString()}] Status: ${service.healthy ? 'Healthy' : 'Unhealthy'}`,
        service.error ? `[${new Date().toISOString()}] Error: ${service.error}` : '',
        `[${new Date().toISOString()}] Last checked: ${service.lastChecked?.toLocaleTimeString() ?? 'Never'}`,
        '',
        'Note: Full logs are available at:',
        `  tail -f logs/${logFileName}`,
        '',
        'Or view in terminal:',
        `  cat logs/${logFileName}`,
      ].filter(Boolean);

      setLogs(mockLogs);
    } catch (error) {
      setLogError(error instanceof Error ? error.message : 'Failed to fetch logs');
    } finally {
      setLoadingLogs(false);
    }
  }, [service]);

  useEffect(() => {
    fetchLogs();

    if (autoRefresh) {
      const interval = setInterval(() => {
        fetchLogs();
      }, 5000); // Refresh every 5 seconds
      return () => clearInterval(interval);
    }
  }, [fetchLogs, autoRefresh]);

  // Map service display names to API service names
  const getApiServiceName = (displayName: string): string | null => {
    const nameMap: Record<string, string> = {
      'Alpaca': 'alpaca',
      'TradeStation': 'tradestation',
      'IB': 'ib',
      'Discount Bank': 'discount_bank',
      'Risk-Free Rate': 'risk_free_rate',
      'Tastytrade': 'tastytrade',
    };
    return nameMap[displayName] || null;
  };

  const handleStartService = async () => {
    const apiName = getApiServiceName(service.name);
    if (!apiName) {
      alert(`${service.name} service control is not available via API.`);
      return;
    }

    try {
      const response = await fetch(`http://localhost:8080/api/v1/services/${apiName}/start`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ force: false }),
      });

      if (response.ok) {
        const data = await response.json();
        console.log(`[Service Control] ✅ ${service.name} started:`, data);
        alert(`✅ ${service.name} started successfully!`);
        onRefresh();
      } else {
        const errorText = await response.text();
        console.error(`[Service Control] ❌ Failed to start ${service.name}:`, errorText);
        alert(`❌ Failed to start ${service.name}: ${errorText}`);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error(`[Service Control] ❌ Error starting ${service.name}:`, error);
      alert(`❌ Error starting ${service.name}: ${errorMessage}`);
    }
  };

  const handleStopService = async () => {
    const apiName = getApiServiceName(service.name);
    if (!apiName) {
      alert(`${service.name} service control is not available via API.`);
      return;
    }

    try {
      const response = await fetch(`http://localhost:8080/api/v1/services/${apiName}/stop`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ force: false }),
      });

      if (response.ok) {
        const data = await response.json();
        console.log(`[Service Control] ✅ ${service.name} stopped:`, data);
        alert(`✅ ${service.name} stopped successfully!`);
        onRefresh();
      } else {
        const errorText = await response.text();
        console.error(`[Service Control] ❌ Failed to stop ${service.name}:`, errorText);
        alert(`❌ Failed to stop ${service.name}: ${errorText}`);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error(`[Service Control] ❌ Error stopping ${service.name}:`, error);
      alert(`❌ Error stopping ${service.name}: ${errorMessage}`);
    }
  };

  const handleEnableService = async () => {
    const apiName = getApiServiceName(service.name);
    if (!apiName) {
      alert(`${service.name} service control is not available via API.`);
      return;
    }

    try {
      const response = await fetch(`http://localhost:8080/api/v1/services/${apiName}/enable`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      });

      if (response.ok) {
        const data = await response.json();
        console.log(`[Service Control] ✅ ${service.name} enabled:`, data);
        alert(`✅ ${service.name} enabled successfully!`);
        onRefresh();
      } else {
        const errorText = await response.text();
        console.error(`[Service Control] ❌ Failed to enable ${service.name}:`, errorText);
        alert(`❌ Failed to enable ${service.name}: ${errorText}`);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error(`[Service Control] ❌ Error enabling ${service.name}:`, error);
      alert(`❌ Error enabling ${service.name}: ${errorMessage}`);
    }
  };

  const handleDisableService = async () => {
    const apiName = getApiServiceName(service.name);
    if (!apiName) {
      alert(`${service.name} service control is not available via API.`);
      return;
    }

    try {
      const response = await fetch(`http://localhost:8080/api/v1/services/${apiName}/disable`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
      });

      if (response.ok) {
        const data = await response.json();
        console.log(`[Service Control] ✅ ${service.name} disabled:`, data);
        alert(`✅ ${service.name} disabled successfully!`);
        onRefresh();
      } else {
        const errorText = await response.text();
        console.error(`[Service Control] ❌ Failed to disable ${service.name}:`, errorText);
        alert(`❌ Failed to disable ${service.name}: ${errorText}`);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      console.error(`[Service Control] ❌ Error disabling ${service.name}:`, error);
      alert(`❌ Error disabling ${service.name}: ${errorMessage}`);
    }
  };

  const handleTestHealth = () => {
    console.log(`[Service Health] Testing ${service.name} on port ${service.port}...`);
    void fetch(`http://localhost:${service.port}/api/health`, {
      method: 'GET',
      signal: AbortSignal.timeout(config.timeout),
    })
      .then((response) => {
        if (response.ok) {
          console.log(`[Service Health] ✅ ${service.name} is healthy`);
          alert(`${service.name} is healthy!`);
        } else {
          console.log(`[Service Health] ❌ ${service.name} returned status ${response.status}`);
          alert(`${service.name} returned status ${response.status}`);
        }
        onRefresh();
      })
      .catch((error: unknown) => {
        const errorMessage = error instanceof Error ? error.message : 'Unknown error';
        console.error(`[Service Health] ❌ ${service.name} health check failed:`, error);
        alert(`Health check failed: ${errorMessage}`);
      });
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()} style={{
        maxWidth: '800px',
        maxHeight: '90vh',
        display: 'flex',
        flexDirection: 'column',
      }}>
        <header className="modal__header">
          <h3>{service.name} Service Configuration</h3>
          <button
            type="button"
            className="modal__close"
            onClick={onClose}
            aria-label="Close"
          >
            ×
          </button>
        </header>

        <div className="modal__body" style={{
          padding: '24px',
          overflow: 'auto',
          flex: 1,
        }}>
          {/* Service Status */}
          <div style={{ marginBottom: '24px' }}>
            <h3 style={{ marginTop: 0, marginBottom: '12px', fontSize: '1.1rem' }}>Service Status</h3>
            <div style={{
              padding: '12px',
              background: 'rgba(148, 163, 184, 0.1)',
              borderRadius: '8px',
              display: 'flex',
              gap: '16px',
              flexWrap: 'wrap',
            }}>
              <div>
                <strong>Status:</strong>{' '}
                <span style={{ color: service.healthy ? '#22c55e' : '#f87171' }}>
                  {service.healthy ? '✅ Healthy' : '❌ Unhealthy'}
                </span>
              </div>
              <div>
                <strong>Port:</strong> {service.port}
              </div>
              {service.enabled !== undefined && (
                <div>
                  <strong>Enabled:</strong>{' '}
                  <span style={{ color: service.enabled ? '#22c55e' : '#9ca3af' }}>
                    {service.enabled ? '✅ Yes' : '🚫 No (Disabled)'}
                  </span>
                </div>
              )}
              {service.running !== undefined && (
                <div>
                  <strong>Running:</strong>{' '}
                  <span style={{ color: service.running ? '#22c55e' : '#f87171' }}>
                    {service.running ? '✅ Yes' : '❌ No'}
                  </span>
                </div>
              )}
              {service.pid && (
                <div>
                  <strong>PID:</strong> {service.pid}
                </div>
              )}
              {service.authenticated !== undefined && (
                <div>
                  <strong>Authenticated:</strong>{' '}
                  <span style={{ color: service.authenticated ? '#22c55e' : '#f87171' }}>
                    {service.authenticated ? '✅ Yes' : '❌ No'}
                  </span>
                </div>
              )}
              {service.attentionRequired && service.attentionRequired !== 'none' && (
                <div style={{ width: '100%', marginTop: '8px' }}>
                  <strong>Attention Required:</strong>{' '}
                  <span style={{
                    color: service.attentionRequired === 'authentication' ? '#f59e0b' :
                           service.attentionRequired === 'credentials' ? '#ef4444' :
                           service.attentionRequired === 'configuration' ? '#3b82f6' :
                           '#f87171',
                    fontWeight: 'bold',
                  }}>
                    {service.attentionRequired === 'authentication' && '🔐 Authentication'}
                    {service.attentionRequired === 'credentials' && '🔑 Credentials'}
                    {service.attentionRequired === 'configuration' && '⚙️ Configuration'}
                    {service.attentionRequired === 'error' && '⚠️ Error'}
                  </span>
                  {service.attentionMessage && (
                    <div style={{ marginTop: '4px', fontSize: '0.9rem', color: '#9ca3af' }}>
                      {service.attentionMessage}
                    </div>
                  )}
                </div>
              )}
              <div>
                <strong>Last Checked:</strong>{' '}
                {service.lastChecked ? service.lastChecked.toLocaleTimeString() : 'Never'}
              </div>
              {service.error && (
                <div style={{ width: '100%', color: '#f87171' }}>
                  <strong>Error:</strong> {service.error}
                </div>
              )}
            </div>
            {service.authenticated === false && service.authUrl && (
              <div style={{
                marginTop: '12px',
                padding: '12px',
                background: 'rgba(251, 191, 36, 0.2)',
                borderRadius: '8px',
                border: '1px solid rgba(251, 191, 36, 0.4)',
              }}>
                <div style={{ marginBottom: '8px', fontWeight: 'bold', color: '#fbbf24' }}>
                  ⚠️ Authentication Required
                </div>
                <div style={{ marginBottom: '8px', fontSize: '0.9rem' }}>
                  This service requires authentication to function properly.
                </div>
                <button
                  onClick={() => {
                    console.log(`[Service Auth] Opening authentication page for ${service.name}: ${service.authUrl}`);
                    if (service.authUrl) {
                      window.open(service.authUrl, '_blank', 'noopener,noreferrer');
                    }
                  }}
                  style={{
                    padding: '8px 16px',
                    background: '#f59e0b',
                    color: 'white',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    fontSize: '0.9rem',
                    fontWeight: 'bold',
                  }}
                >
                  🔐 Open Authentication Page
                </button>
              </div>
            )}
          </div>

          {/* Service Controls */}
          <div style={{ marginBottom: '24px' }}>
            <h3 style={{ marginTop: 0, marginBottom: '12px', fontSize: '1.1rem' }}>Service Controls</h3>
            <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
              <button
                onClick={handleTestHealth}
                style={{
                  padding: '8px 16px',
                  background: '#3b82f6',
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: 'pointer',
                  fontSize: '0.9rem',
                }}
              >
                🔍 Test Health
              </button>
              <button
                onClick={handleStartService}
                disabled={service.healthy || service.enabled === false}
                style={{
                  padding: '8px 16px',
                  background: (service.healthy || service.enabled === false) ? '#6b7280' : '#22c55e',
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: (service.healthy || service.enabled === false) ? 'not-allowed' : 'pointer',
                  fontSize: '0.9rem',
                  opacity: (service.healthy || service.enabled === false) ? 0.5 : 1,
                }}
                title={service.enabled === false ? 'Service is disabled' : undefined}
              >
                ▶ Start Service
              </button>
              <button
                onClick={handleStopService}
                disabled={!service.healthy || service.enabled === false}
                style={{
                  padding: '8px 16px',
                  background: (!service.healthy || service.enabled === false) ? '#6b7280' : '#ef4444',
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: (!service.healthy || service.enabled === false) ? 'not-allowed' : 'pointer',
                  fontSize: '0.9rem',
                  opacity: (!service.healthy || service.enabled === false) ? 0.5 : 1,
                }}
                title={service.enabled === false ? 'Service is disabled' : undefined}
              >
                ⏹ Stop Service
              </button>
              {service.enabled !== undefined && (
                <>
                  {service.enabled ? (
                    <button
                      onClick={handleDisableService}
                      style={{
                        padding: '8px 16px',
                        background: '#f59e0b',
                        color: 'white',
                        border: 'none',
                        borderRadius: '6px',
                        cursor: 'pointer',
                        fontSize: '0.9rem',
                      }}
                    >
                      🚫 Disable Service
                    </button>
                  ) : (
                    <button
                      onClick={handleEnableService}
                      style={{
                        padding: '8px 16px',
                        background: '#22c55e',
                        color: 'white',
                        border: 'none',
                        borderRadius: '6px',
                        cursor: 'pointer',
                        fontSize: '0.9rem',
                      }}
                    >
                      ✅ Enable Service
                    </button>
                  )}
                </>
              )}
              <button
                onClick={onRefresh}
                style={{
                  padding: '8px 16px',
                  background: '#6366f1',
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: 'pointer',
                  fontSize: '0.9rem',
                }}
              >
                🔄 Refresh Status
              </button>
            </div>
          </div>

          {/* Configuration */}
          <div style={{ marginBottom: '24px' }}>
            <h3 style={{ marginTop: 0, marginBottom: '12px', fontSize: '1.1rem' }}>Configuration</h3>
            <div style={{
              padding: '12px',
              background: 'rgba(148, 163, 184, 0.1)',
              borderRadius: '8px',
            }}>
              <div style={{ marginBottom: '12px' }}>
                <label style={{ display: 'block', marginBottom: '4px', fontSize: '0.9rem' }}>
                  Port:
                </label>
                <input
                  type="number"
                  value={config.port}
                  onChange={(e) => setConfig({ ...config, port: parseInt(e.target.value) || 0 })}
                  style={{
                    width: '100%',
                    padding: '8px',
                    background: 'rgba(15, 23, 42, 0.8)',
                    border: '1px solid rgba(148, 163, 184, 0.3)',
                    borderRadius: '4px',
                    color: '#e2e8f0',
                    fontSize: '0.9rem',
                  }}
                  disabled
                  title="Port configuration requires service restart"
                />
                <small style={{ color: '#9ca3af', fontSize: '0.8rem' }}>
                  Port changes require service restart
                </small>
              </div>
              <div style={{ marginBottom: '12px' }}>
                <label htmlFor="health-check-interval" style={{ display: 'block', marginBottom: '4px', fontSize: '0.9rem' }}>
                  Health Check Interval (ms):
                </label>
                <input
                  id="health-check-interval"
                  type="number"
                  value={config.healthCheckInterval}
                  onChange={(e) => setConfig({ ...config, healthCheckInterval: parseInt(e.target.value) || 10000 })}
                  style={{
                    width: '100%',
                    padding: '8px',
                    background: 'rgba(15, 23, 42, 0.8)',
                    border: '1px solid rgba(148, 163, 184, 0.3)',
                    borderRadius: '4px',
                    color: '#e2e8f0',
                    fontSize: '0.9rem',
                  }}
                />
              </div>
              <div>
                <label htmlFor="health-check-timeout" style={{ display: 'block', marginBottom: '4px', fontSize: '0.9rem' }}>
                  Health Check Timeout (ms):
                </label>
                <input
                  id="health-check-timeout"
                  type="number"
                  value={config.timeout}
                  onChange={(e) => setConfig({ ...config, timeout: parseInt(e.target.value) || 2000 })}
                  style={{
                    width: '100%',
                    padding: '8px',
                    background: 'rgba(15, 23, 42, 0.8)',
                    border: '1px solid rgba(148, 163, 184, 0.3)',
                    borderRadius: '4px',
                    color: '#e2e8f0',
                    fontSize: '0.9rem',
                  }}
                />
              </div>
            </div>
          </div>

          {/* Logs */}
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
              <h3 style={{ margin: 0, fontSize: '1.1rem' }}>Service Logs</h3>
              <label style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '0.9rem', cursor: 'pointer' }}>
                <input
                  type="checkbox"
                  checked={autoRefresh}
                  onChange={(e) => setAutoRefresh(e.target.checked)}
                />
                Auto-refresh (5s)
              </label>
            </div>
            <div style={{
              padding: '12px',
              background: 'rgba(15, 23, 42, 0.9)',
              borderRadius: '8px',
              border: '1px solid rgba(148, 163, 184, 0.2)',
              maxHeight: '300px',
              overflow: 'auto',
              fontFamily: 'monospace',
              fontSize: '0.85rem',
              lineHeight: '1.5',
            }}>
              {loadingLogs ? (
                <div style={{ color: '#9ca3af' }}>Loading logs...</div>
              ) : logError ? (
                <div style={{ color: '#f87171' }}>Error: {logError}</div>
              ) : (
                <pre style={{ margin: 0, color: '#e2e8f0', whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
                  {logs.length > 0 ? logs.join('\n') : 'No logs available'}
                </pre>
              )}
            </div>
            <div style={{ marginTop: '8px', fontSize: '0.8rem', color: '#9ca3af' }}>
              <strong>Note:</strong> Full logs are available in the terminal:
              <br />
              <code style={{ background: 'rgba(148, 163, 184, 0.1)', padding: '2px 6px', borderRadius: '4px' }}>
                tail -f logs/{getLogFileName(service.name)}
              </code>
            </div>
          </div>
        </div>

        <footer className="modal__footer">
          <button
            type="button"
            className="btn btn--primary"
            onClick={onClose}
          >
            Close
          </button>
        </footer>
      </div>
    </div>
  );
}
