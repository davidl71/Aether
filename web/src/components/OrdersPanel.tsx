import { useState, useCallback } from 'react';
import type { TimelineEvent } from '../types/snapshot';

interface OrdersPanelProps {
  orders: TimelineEvent[];
  onCancelOrder?: (orderId: string) => void;
  apiBaseUrl?: string;
}

export function OrdersPanel({ orders, onCancelOrder, apiBaseUrl }: OrdersPanelProps) {
  const [cancellingOrderIds, setCancellingOrderIds] = useState<Set<string>>(new Set());

  const handleCancelOrder = useCallback(async (orderId: string) => {
    if (cancellingOrderIds.has(orderId)) {
      return; // Already cancelling
    }

    // Show confirmation
    if (!window.confirm(`Cancel order ${orderId}?`)) {
      return;
    }

    setCancellingOrderIds((prev) => new Set(prev).add(orderId));

    try {
      if (onCancelOrder) {
        onCancelOrder(orderId);
      } else {
        // Fallback: call API directly
        const baseUrl = apiBaseUrl || 'http://127.0.0.1:8080';
        const response = await fetch(`${baseUrl}/api/v1/orders/cancel`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ order_id: orderId })
        });

        if (!response.ok) {
          throw new Error(`Failed to cancel order: ${response.statusText}`);
        }

        console.log(`Order ${orderId} cancelled successfully`);
      }
    } catch (error) {
      console.error('Error cancelling order:', error);
      alert(`Failed to cancel order: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setCancellingOrderIds((prev) => {
        const next = new Set(prev);
        next.delete(orderId);
        return next;
      });
    }
  }, [onCancelOrder, apiBaseUrl, cancellingOrderIds]);

  // Extract order ID from order text (heuristic - may need adjustment based on actual format)
  const extractOrderId = (orderText: string): string | null => {
    // Try to find order ID patterns in the text
    const idMatch = orderText.match(/order[_\s]*#?(\d+)/i) || orderText.match(/id[:\s]+(\d+)/i);
    return idMatch ? idMatch[1] : null;
  };

  return (
    <div className="panel panel--fill">
      <div className="panel__header">
        <div>
          <h2>Recent Orders</h2>
          <p>Live order events from the strategy gateway.</p>
        </div>
      </div>
      <ul className="timeline" aria-live="polite">
        {orders.map((order) => {
          const orderId = extractOrderId(order.text);
          const canCancel = orderId && (order.severity === 'info' || order.severity === 'success');
          const isCancelling = orderId ? cancellingOrderIds.has(orderId) : false;

          return (
            <li key={order.timestamp} className={`timeline__item timeline__item--${order.severity}`}>
              <span className="timeline__time">{new Date(order.timestamp).toLocaleTimeString()}</span>
              <span className="timeline__text">{order.text}</span>
              {canCancel && (onCancelOrder || apiBaseUrl) && (
                <button
                  type="button"
                  className="btn btn--small btn--secondary"
                  onClick={() => orderId && handleCancelOrder(orderId)}
                  disabled={isCancelling}
                  style={{ marginLeft: '8px' }}
                  title={`Cancel order ${orderId}`}
                >
                  {isCancelling ? 'Cancelling...' : 'Cancel'}
                </button>
              )}
            </li>
          );
        })}
        {orders.length === 0 && <li className="timeline__item">No orders yet.</li>}
      </ul>
    </div>
  );
}
