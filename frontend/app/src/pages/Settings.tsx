import type React from "react"
import { Copy, ExternalLink, RefreshCw } from "lucide-react"
import { useWallet } from "../contexts/WalletContext"

const Settings: React.FC = () => {
    const { icp, evm } = useWallet()

    const copyToClipboard = (text: string) => {
        navigator.clipboard.writeText(text)
        // we could add a toast notification here
    }

    return (
        <div className="settings">
            <h1>Settings</h1>

            <div className="settings-sections">
                <section className="settings-section">
                    <h2>Identity & Wallets</h2>

                    <div className="wallet-details">
                        <div className="wallet-detail-card">
                            <h3>Internet Computer</h3>
                            {icp ? (
                                <div className="wallet-info">
                                    <div className="info-row">
                                        <span>Principal ID:</span>
                                        <div className="copyable">
                                            <span className="address">rdmx6-jaaaa-aaaah-qcaiq-cai</span>
                                            <button onClick={() => copyToClipboard("rdmx6-jaaaa-aaaah-qcaiq-cai")}>
                                                <Copy size={16} />
                                            </button>
                                        </div>
                                    </div>
                                    <div className="info-row">
                                        <span>Cycle Balance:</span>
                                        <span>1.2T cycles</span>
                                    </div>
                                    <div className="info-row">
                                        <span>ICP Balance:</span>
                                        <span>15.4 ICP</span>
                                    </div>
                                </div>
                            ) : (
                                <p className="not-connected">Not connected</p>
                            )}
                        </div>

                        <div className="wallet-detail-card">
                            <h3>Ethereum</h3>
                            {evm ? (
                                <div className="wallet-info">
                                    <div className="info-row">
                                        <span>Address:</span>
                                        <div className="copyable">
                                            <span className="address">0x742d35Cc6634C0532925a3b8D</span>
                                            <button onClick={() => copyToClipboard("0x742d35Cc6634C0532925a3b8D")}>
                                                <Copy size={16} />
                                            </button>
                                        </div>
                                    </div>
                                    <div className="info-row">
                                        <span>Network:</span>
                                        <span>Ethereum Mainnet</span>
                                    </div>
                                    <div className="info-row">
                                        <span>ETH Balance:</span>
                                        <span>0.25 ETH</span>
                                    </div>
                                </div>
                            ) : (
                                <p className="not-connected">Not connected</p>
                            )}
                        </div>

                        {/* <div className="wallet-detail-card">
                            <h3>Bitcoin</h3>
                            {bitcoin ? (
                                <div className="wallet-info">
                                    <div className="info-row">
                                        <span>Address:</span>
                                        <div className="copyable">
                                            <span className="address">bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh</span>
                                            <button onClick={() => copyToClipboard("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")}>
                                                <Copy size={16} />
                                            </button>
                                        </div>
                                    </div>
                                    <div className="info-row">
                                        <span>Network:</span>
                                        <span>Bitcoin Mainnet</span>
                                    </div>
                                    <div className="info-row">
                                        <span>BTC Balance:</span>
                                        <span>0.00125 BTC</span>
                                    </div>
                                </div>
                            ) : (
                                <p className="not-connected">Not connected</p>
                            )}
                        </div> */}
                    </div>
                </section>

                <section className="settings-section">
                    <h2>Network Information</h2>

                    <div className="network-info">
                        <div className="network-card">
                            <h3>Internet Computer</h3>
                            <div className="info-row">
                                <span>Network:</span>
                                <span>IC Mainnet</span>
                            </div>
                            <div className="info-row">
                                <span>Orderbook Canister:</span>
                                <div className="copyable">
                                    <span className="address">rdmx6-jaaaa-aaaah-qcaiq-cai</span>
                                    <button onClick={() => copyToClipboard("rdmx6-jaaaa-aaaah-qcaiq-cai")}>
                                        <Copy size={16} />
                                    </button>
                                </div>
                            </div>
                            <div className="info-row">
                                <span>Factory Canister:</span>
                                <div className="copyable">
                                    <span className="address">rrkah-fqaaa-aaaah-qcaiq-cai</span>
                                    <button onClick={() => copyToClipboard("rrkah-fqaaa-aaaah-qcaiq-cai")}>
                                        <Copy size={16} />
                                    </button>
                                </div>
                            </div>
                        </div>

                        <div className="network-card">
                            <h3>Ethereum</h3>
                            <div className="info-row">
                                <span>Network:</span>
                                <span>Mainnet</span>
                            </div>
                            <div className="info-row">
                                <span>Chain ID:</span>
                                <span>1</span>
                            </div>
                            <div className="info-row">
                                <span>RPC URL:</span>
                                <span>https://mainnet.infura.io/v3/...</span>
                            </div>
                        </div>
                    </div>
                </section>

                <section className="settings-section">
                    <h2>Preferences</h2>

                    <div className="preferences">
                        <div className="preference-item">
                            <label>
                                <input type="checkbox" defaultChecked />
                                Enable notifications for order updates
                            </label>
                        </div>
                        <div className="preference-item">
                            <label>
                                <input type="checkbox" defaultChecked />
                                Auto-refresh order status
                            </label>
                        </div>
                        <div className="preference-item">
                            <label>
                                <input type="checkbox" />
                                Advanced trading features
                            </label>
                        </div>
                    </div>
                </section>

                <section className="settings-section">
                    <h2>Actions</h2>

                    <div className="action-buttons">
                        <button className="action-button">
                            <RefreshCw size={18} />
                            Refresh All Balances
                        </button>
                        <button className="action-button">
                            <ExternalLink size={18} />
                            View on IC Dashboard
                        </button>
                    </div>
                </section>
            </div>
        </div>
    )
}

export default Settings
