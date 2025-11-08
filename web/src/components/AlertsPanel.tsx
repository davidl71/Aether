import type { TimelineEvent } from '../types/snapshot';

interface AlertsPanelProps {
  alerts: TimelineEvent[];
}

export function AlertsPanel({ alerts }: AlertsPanelProps) {
  return (
    <div className="panel panel--fill">
      <div className="panel__header">
        <div>
          <h2>Alerts</h2>
          <p>Gateway notifications, risk warnings, and system status.</p>
        </div>
      </div>
      <div className="alerts" role="log" aria-live="polite">
        {alerts.map((alert) => (
          <div key={alert.timestamp} className={`alerts__item alerts__item--${alert.severity}`}>
            <span className="alerts__time">{new Date(alert.timestamp).toLocaleTimeString()}</span>
            <span className="alerts__text">{alert.text}</span>
          </div>
        ))}
        {alerts.length === 0 && <div className="alerts__item">All clear.</div>}
      </div>
    </div>
  );
}
