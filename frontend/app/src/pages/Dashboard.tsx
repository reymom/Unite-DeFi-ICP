import type React from "react"
import { Link } from "react-router-dom"
import { Plus, Activity, Clock, AlertCircle } from "lucide-react"
import { useOrders } from "../hooks/useOrders"

const Dashboard: React.FC = () => {
    const { orders, loading, error } = useOrders()

    const stats = {
        totalVolume: orders.reduce((sum, o) => sum + Number(o.escrow_params.amount), 0),
    }

    return (
        <div className="dashboard">
            <div className="dashboard-header">
                <h1>Dashboard</h1>
                <Link to="/swap" className="create-order-btn">
                    <Plus size={18} />
                    New Order
                </Link>
            </div>

            <div className="stats-grid">
                <div className="stat-card">
                    <div className="stat-icon">
                        <Clock size={24} />
                    </div>
                    <div className="stat-content">
                        <h3>Open Orders</h3>
                        <p className="stat-value">{orders.length}</p>
                    </div>
                </div>

                <div className="stat-card">
                    <div className="stat-icon">
                        <Activity size={24} />
                    </div>
                    <div className="stat-content">
                        <h3>Total Volume</h3>
                        <p className="stat-value">{(stats.totalVolume / 1e8).toFixed(2)} ICP</p>
                    </div>
                </div>
            </div>

            <div className="dashboard-content">
                <div className="orders-section">
                    <h2>Recent Orders</h2>
                    {loading ? (
                        <div className="loading">Loading orders...</div>
                    ) : error ? (
                        <div className="error">
                            <AlertCircle size={20} />
                            Failed to load orders: {error}
                        </div>
                    ) : orders.length === 0 ? (
                        <div className="empty-state">
                            <p>No orders yet. Create your first swap order!</p>
                            <Link to="/swap" className="btn-primary">
                                Create Order
                            </Link>
                        </div>
                    ) : (
                        <div className="orders-list">
                            {orders.slice(0, 5).map((order, index) => (
                                <div key={index} className="order-card">
                                    <div className="order-header">
                                        <span className="order-id">#{order.order_hash.slice(0, 8)}</span>
                                    </div>
                                    <div className="order-details">
                                        <p>Amount: {(Number(order.escrow_params.amount) / 1e8).toFixed(4)} ICP</p>
                                        <p>Created: {new Date(Number(order.auction_start_at) * 1000).toLocaleDateString()}</p>
                                    </div>
                                    <Link to={`/trade/${order.order_hash}`} className="view-details">
                                        View Details
                                    </Link>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </div>
        </div>
    )
}

export default Dashboard
