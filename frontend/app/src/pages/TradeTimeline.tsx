import type React from "react"
import { useParams } from "react-router-dom"
import { CheckCircle, Clock, AlertCircle } from "lucide-react"

const TradeTimeline: React.FC = () => {
    const { orderId } = useParams<{ orderId: string }>()

    const steps = [
        {
            id: "order_created",
            title: "Order Created",
            description: "Swap order intent submitted to orderbook",
            status: "completed",
            timestamp: "2024-01-15 10:30:00",
        },
        {
            id: "escrow_created",
            title: "Escrow Created",
            description: "Factory canister created escrow contract",
            status: "completed",
            timestamp: "2024-01-15 10:31:15",
        },
        {
            id: "funds_locked",
            title: "Funds Locked",
            description: "Source funds locked in escrow with hashlock",
            status: "completed",
            timestamp: "2024-01-15 10:32:00",
        },
        {
            id: "auction_started",
            title: "Dutch Auction Started",
            description: "Relayers can now bid on this order",
            status: "active",
            timestamp: "2024-01-15 10:33:00",
        },
        {
            id: "resolver_selected",
            title: "Resolver Selected",
            description: "Best resolver chosen from auction",
            status: "pending",
            timestamp: null,
        },
        {
            id: "cross_chain_tx",
            title: "Cross-chain Transaction",
            description: "Resolver executes transaction on destination chain",
            status: "pending",
            timestamp: null,
        },
        {
            id: "secret_revealed",
            title: "Secret Revealed",
            description: "Resolver reveals secret to claim source funds",
            status: "pending",
            timestamp: null,
        },
        {
            id: "trade_completed",
            title: "Trade Completed",
            description: "All parties have received their funds",
            status: "pending",
            timestamp: null,
        },
    ]

    const getStepIcon = (status: string) => {
        switch (status) {
            case "completed":
                return <CheckCircle className="step-icon completed" size={24} />
            case "active":
                return <Clock className="step-icon active" size={24} />
            case "failed":
                return <AlertCircle className="step-icon failed" size={24} />
            default:
                return <div className="step-icon pending" />
        }
    }

    return (
        <div className="trade-timeline">
            <div className="timeline-header">
                <h1>Trade Progress</h1>
                <div className="order-info">
                    <span className="order-id">Order #{orderId?.slice(0, 8)}...</span>
                    <span className="order-status active">In Progress</span>
                </div>
            </div>

            <div className="timeline-container">
                <div className="timeline">
                    {steps.map((step, index) => (
                        <div key={step.id} className={`timeline-step ${step.status}`}>
                            <div className="step-connector">
                                {getStepIcon(step.status)}
                                {index < steps.length - 1 && (
                                    <div className={`connector-line ${step.status === "completed" ? "completed" : ""}`} />
                                )}
                            </div>

                            <div className="step-content">
                                <div className="step-header">
                                    <h3 className="step-title">{step.title}</h3>
                                    {step.timestamp && <span className="step-timestamp">{step.timestamp}</span>}
                                </div>
                                <p className="step-description">{step.description}</p>

                                {step.status === "active" && (
                                    <div className="step-progress">
                                        <div className="progress-bar">
                                            <div className="progress-fill" style={{ width: "60%" }} />
                                        </div>
                                        <span className="progress-text">Waiting for resolver...</span>
                                    </div>
                                )}
                            </div>
                        </div>
                    ))}
                </div>

                <div className="trade-details">
                    <div className="details-card">
                        <h3>Trade Details</h3>
                        <div className="detail-row">
                            <span>From:</span>
                            <span>2.5 ICP</span>
                        </div>
                        <div className="detail-row">
                            <span>To:</span>
                            <span>0.001 ETH</span>
                        </div>
                        <div className="detail-row">
                            <span>Rate:</span>
                            <span>1 ICP = 0.0004 ETH</span>
                        </div>
                        <div className="detail-row">
                            <span>Safety Deposit:</span>
                            <span>0.5 ICP</span>
                        </div>
                        <div className="detail-row">
                            <span>Timelock:</span>
                            <span>1 hour</span>
                        </div>
                    </div>

                    <div className="details-card">
                        <h3>Participants</h3>
                        <div className="participant">
                            <span className="role">Maker:</span>
                            <span className="address">rdmx6-jaaaa-aaaah-qcaiq-cai</span>
                        </div>
                        <div className="participant">
                            <span className="role">Resolver:</span>
                            <span className="address">Pending selection...</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    )
}

export default TradeTimeline
