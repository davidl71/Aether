import type { TimelineEvent } from '../types/snapshot';

interface OrdersPanelProps {
  orders: TimelineEvent[];
}

export function OrdersPanel({ orders }: OrdersPanelProps) {
  return (
    <div className="panel panel--fill">
      <div className="panel__header">
        <div>
          <h2>Recent Orders</h2>
          <p>Live order events from the strategy gateway.</p>
        </div>
      </div>
      <ul className="timeline" aria-live="polite">
        {orders.map((order) => (
          <li key={order.timestamp} className={`timeline__item timeline__item--${order.severity}`}>
            <span className="timeline__time">{new Date(order.timestamp).toLocaleTimeString()}</span>
            <span className="timeline__text">{order.text}</span>
          </li>
        ))}
        {orders.length === 0 && <li className="timeline__item">No orders yet.</li>}
      </ul>
    </div>
  );
}
