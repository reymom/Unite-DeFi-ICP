import type React from "react"
import { createContext, useContext, useState, type ReactNode } from "react"
import type { Order } from "declarations/orderbook/orderbook.did"

interface OrderContextType {
    orders: Order[]
    loading: boolean
    error: string | null
    refreshOrders: () => Promise<void>
    createOrder: (orderData: any) => Promise<void>
}

const OrderContext = createContext<OrderContextType | undefined>(undefined)

export const useOrderContext = () => {
    const context = useContext(OrderContext)
    if (!context) {
        throw new Error("useOrderContext must be used within an OrderProvider")
    }
    return context
}

interface OrderProviderProps {
    children: ReactNode
}

export const OrderProvider: React.FC<OrderProviderProps> = ({ children }) => {
    const [orders, setOrders] = useState<Order[]>([])
    const [loading, setLoading] = useState(false)
    const [error, setError] = useState<string | null>(null)

    const refreshOrders = async () => {
        setLoading(true)
        setError(null)
        try {
            // Import orderbook canister and fetch orders
            const { orderbook } = await import("declarations/orderbook")
            const fetchedOrders = await orderbook.list_orders()
            setOrders(fetchedOrders)
        } catch (err) {
            setError(err instanceof Error ? err.message : "Failed to fetch orders")
        } finally {
            setLoading(false)
        }
    }

    const createOrder = async (orderData: any) => {
        setLoading(true)
        try {
            // Implementation would go here
            console.log("Creating order with data:", orderData)
            await refreshOrders()
        } catch (err) {
            setError(err instanceof Error ? err.message : "Failed to create order")
            throw err
        } finally {
            setLoading(false)
        }
    }

    return (
        <OrderContext.Provider
            value={{
                orders,
                loading,
                error,
                refreshOrders,
                createOrder,
            }}
        >
            {children}
        </OrderContext.Provider>
    )
}