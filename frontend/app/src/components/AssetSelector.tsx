import type React from "react"
import { useState } from "react"
import { ChevronDown } from "lucide-react"

interface Asset {
    symbol: string
    network: string
    balance: string
    icon?: string
}

interface AssetSelectorProps {
    asset: Asset
    onAssetChange: (asset: Asset) => void
}

const AssetSelector: React.FC<AssetSelectorProps> = ({ asset, onAssetChange }) => {
    const [isOpen, setIsOpen] = useState(false)

    const availableAssets: Asset[] = [
        { symbol: "ICP", network: "Internet Computer", balance: "15.4", icon: "🔵" },
        { symbol: "ETH", network: "Ethereum", balance: "0.25", icon: "⟠" },
        { symbol: "BTC", network: "Bitcoin", balance: "0.00125", icon: "₿" },
        { symbol: "USDC", network: "Ethereum", balance: "1250.0", icon: "💵" },
    ]

    return (
        <div className="asset-selector">
            <button type="button" className="asset-button" onClick={() => setIsOpen(!isOpen)}>
                <div className="asset-info">
                    <span className="asset-icon">{asset.icon || "🔵"}</span>
                    <div className="asset-details">
                        <span className="asset-symbol">{asset.symbol}</span>
                        <span className="asset-network">{asset.network}</span>
                    </div>
                </div>
                <ChevronDown size={16} className={`chevron ${isOpen ? "open" : ""}`} />
            </button>

            {isOpen && (
                <div className="asset-dropdown">
                    {availableAssets.map((availableAsset) => (
                        <button
                            key={`${availableAsset.symbol}-${availableAsset.network}`}
                            type="button"
                            className={`asset-option ${asset.symbol === availableAsset.symbol ? "selected" : ""}`}
                            onClick={() => {
                                onAssetChange(availableAsset)
                                setIsOpen(false)
                            }}
                        >
                            <div className="asset-info">
                                <span className="asset-icon">{availableAsset.icon}</span>
                                <div className="asset-details">
                                    <span className="asset-symbol">{availableAsset.symbol}</span>
                                    <span className="asset-network">{availableAsset.network}</span>
                                </div>
                            </div>
                            <span className="asset-balance">{availableAsset.balance}</span>
                        </button>
                    ))}
                </div>
            )}
        </div>
    )
}

export default AssetSelector
