import { useState, useEffect, useCallback } from "react";
import type { Order } from "declarations/orderbook/orderbook.did";

export const useOrders = (pollInterval = 5000) => {
  const [orders, setOrders] = useState<Order[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchOrders = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const { orderbook } = await import("declarations/orderbook");
      const fetchedOrders = await orderbook.list_orders();
      setOrders(fetchedOrders);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch orders");
    } finally {
      setLoading(false);
    }
  }, []);

  const createOrder = useCallback(
    async (orderData: any) => {
      setLoading(true);
      try {
        // Your order creation logic here
        console.log("Creating order:", orderData);
        await fetchOrders(); // Refresh after creation
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to create order");
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [fetchOrders]
  );

  useEffect(() => {
    fetchOrders();
    const interval = setInterval(fetchOrders, pollInterval);
    return () => clearInterval(interval);
  }, [fetchOrders, pollInterval]);

  return {
    orders,
    loading,
    error,
    refresh: fetchOrders,
    createOrder,
  };
};
