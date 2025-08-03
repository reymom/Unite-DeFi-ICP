import React, { useState } from 'react';
import { ChevronDown, Wallet } from "lucide-react"
import { useWallet } from "../contexts/WalletContext"

const WalletConnector: React.FC = () => {
    const [open, setOpen] = useState(false);
    const {
        icp: { wallet: icpWallet, connect: connectIcp, disconnect: discIcp, loading: loadingIcp },
        evm: { connected: evmConnected, connect: connectEvm, disconnect: discEvm, address },
    } = useWallet();

    const has = icpWallet || evmConnected;

    return (
        <div className="wallet-connector">
            <button className={`wallet-button ${has ? 'connected' : ''}`} onClick={() => setOpen(!open)}>
                <Wallet size={18} />
                {has ? 'Wallets' : 'Connect'}
                <ChevronDown size={16} className={open ? 'rotate-180' : ''} />
            </button>

            {open && (
                <div className="wallet-dropdown">
                    {/* ICP */}
                    <button
                        disabled={loadingIcp}
                        onClick={() => (icpWallet ? discIcp() : connectIcp())}
                        className={icpWallet ? 'connected' : ''}
                    >
                        🪙 Internet Identity
                        <span>{icpWallet ? 'Disconnect' : 'Connect'}</span>
                    </button>

                    {/* EVM */}
                    <button
                        onClick={() => (evmConnected ? discEvm() : connectEvm())}
                        className={evmConnected ? 'connected' : ''}
                    >
                        🦊 EVM Wallet
                        <span>
                            {evmConnected
                                ? address!.slice(0, 6) + '…' + address!.slice(-4)
                                : 'Connect'}
                        </span>
                    </button>
                </div>
            )}
        </div>
    );
};

export default WalletConnector;