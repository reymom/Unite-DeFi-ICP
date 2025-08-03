import React, { useState, useMemo } from 'react';
import { ArrowDown, Settings, RefreshCw, Info } from 'lucide-react';
import { useWallet } from '../contexts/WalletContext';
import { useOrders } from '../hooks/useOrders';
import AssetSelector from '../components/AssetSelector';
import PriceCurveEditor from '../components/PriceCurveEditor';

/* --- helper types --- */
type Chain = 'ICP' | 'EVM';
interface Asset {
    symbol: string;
    network: Chain;
    balance: string;
}

/* --- component --- */
const NewOrder: React.FC = () => {
    const { icp, evm } = useWallet();
    const { createOrder, loading } = useOrders();

    const [formData, setFormData] = useState({
        fromAsset: { symbol: 'ICP', network: 'ICP', balance: '0' } as Asset,
        toAsset: { symbol: 'ETH', network: 'EVM', balance: '0' } as Asset,
        fromAmount: '',
        toAmount: '',
        timelock: '3600',
        safetyDeposit: '0.5',
        slippage: '1.0',
        destinationAddress: '', // principal or EVM address
    });

    const [showAdvanced, setShowAdvanced] = useState(false);
    const [priceCurve, setPriceCurve] = useState([
        { time_offset_secs: BigInt(0), price_multiplier: 1.0 },
        { time_offset_secs: BigInt(300), price_multiplier: 0.95 },
        { time_offset_secs: BigInt(600), price_multiplier: 0.9 },
    ]);

    /* ---- derived state ---- */
    const fromChain: Chain = formData.fromAsset.network;
    const toChain: Chain = formData.toAsset.network;

    const hasSourceWallet =
        (fromChain === 'ICP' && !!icp.wallet) ||
        (fromChain === 'EVM' && evm.connected);

    const isFormValid =
        formData.fromAmount &&
        formData.toAmount &&
        hasSourceWallet &&
        formData.destinationAddress;

    /* ---- handlers ---- */
    const swapAssets = () =>
        setFormData((p) => ({
            ...p,
            fromAsset: p.toAsset,
            toAsset: p.fromAsset,
            fromAmount: p.toAmount,
            toAmount: p.fromAmount,
            destinationAddress: '', // reset
        }));

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!hasSourceWallet) {
            alert(
                `Please connect your ${fromChain === 'ICP' ? 'ICP' : 'EVM'
                } wallet first`
            );
            return;
        }

        try {
            await createOrder({
                amount: formData.fromAmount,
                fromAsset: formData.fromAsset,
                toAsset: formData.toAsset,
                timelock: formData.timelock,
                safetyDeposit: formData.safetyDeposit,
                priceCurve,
                destinationAddress: formData.destinationAddress,
            });
            alert('Order created successfully!');
            // TODO reset form or redirect
        } catch (err) {
            console.error('Order creation failed', err);
            alert('Failed. See console.');
        }
    };

    /* ---- UI ---- */
    return (
        <div className="new-order">
            <div className="swap-container">
                {/* --- header --- */}
                <div className="swap-header">
                    <div className="swap-tabs">
                        <button className="tab active">Swap</button>
                        <button className="tab">Limit</button>
                    </div>
                    <div className="swap-actions">
                        <button
                            className="icon-button"
                            onClick={() => setShowAdvanced((v) => !v)}
                        >
                            <Settings size={18} />
                        </button>
                        <button className="icon-button">
                            <RefreshCw size={18} />
                        </button>
                    </div>
                </div>

                {/* --- form --- */}
                <form onSubmit={handleSubmit} className="swap-form">
                    {/* FROM section */}
                    <div className="swap-section">
                        <div className="section-header">
                            <span>You pay</span>
                            <span className="balance">
                                Balance: {formData.fromAsset.balance}
                            </span>
                        </div>
                        <div className="asset-input">
                            <AssetSelector
                                asset={formData.fromAsset}
                                onAssetChange={(asset) =>
                                    setFormData((p) => ({
                                        ...p,
                                        fromAsset: {
                                            ...asset,
                                            network: asset.symbol === 'ICP' ? 'ICP' : 'EVM',
                                        },
                                    }))
                                }
                            />
                            <input
                                type="number"
                                placeholder="0"
                                value={formData.fromAmount}
                                onChange={(e) =>
                                    setFormData((p) => ({ ...p, fromAmount: e.target.value }))
                                }
                                className="amount-input"
                            />
                        </div>
                    </div>

                    {/* swap arrow */}
                    <div className="swap-divider">
                        <button type="button" className="swap-button" onClick={swapAssets}>
                            <ArrowDown size={20} />
                        </button>
                    </div>

                    {/* TO section */}
                    <div className="swap-section">
                        <div className="section-header">
                            <span>You receive</span>
                            <span className="balance">
                                Balance: {formData.toAsset.balance}
                            </span>
                        </div>
                        <div className="asset-input">
                            <AssetSelector
                                asset={formData.toAsset}
                                onAssetChange={(asset) =>
                                    setFormData((p) => ({
                                        ...p,
                                        toAsset: {
                                            ...asset,
                                            network: asset.symbol === 'ICP' ? 'ICP' : 'EVM',
                                        },
                                    }))
                                }
                            />
                            <input
                                type="number"
                                placeholder="0"
                                value={formData.toAmount}
                                onChange={(e) =>
                                    setFormData((p) => ({ ...p, toAmount: e.target.value }))
                                }
                                className="amount-input"
                            />
                        </div>
                    </div>

                    {/* destination address */}
                    <div className="setting-group mt-3">
                        <label>
                            Destination {toChain === 'ICP' ? 'Principal' : 'EVM address'}
                            <input
                                type="text"
                                placeholder={
                                    toChain === 'ICP'
                                        ? 'aaaaa-aa'
                                        : '0x1234…'
                                }
                                value={formData.destinationAddress}
                                onChange={(e) =>
                                    setFormData((p) => ({
                                        ...p,
                                        destinationAddress: e.target.value,
                                    }))
                                }
                            />
                        </label>
                    </div>

                    {/* advanced settings */}
                    {showAdvanced && (
                        <div className="advanced-settings">
                            <h3>Advanced Settings</h3>

                            <label>
                                Timelock (seconds)
                                <input
                                    type="number"
                                    value={formData.timelock}
                                    onChange={(e) =>
                                        setFormData((p) => ({ ...p, timelock: e.target.value }))
                                    }
                                />
                            </label>

                            {fromChain === 'ICP' && (
                                <label>
                                    Safety Deposit (ICP)
                                    <input
                                        type="number"
                                        step="0.1"
                                        value={formData.safetyDeposit}
                                        onChange={(e) =>
                                            setFormData((p) => ({
                                                ...p,
                                                safetyDeposit: e.target.value,
                                            }))
                                        }
                                    />
                                </label>
                            )}

                            <label>
                                Max Slippage (%)
                                <input
                                    type="number"
                                    step="0.1"
                                    value={formData.slippage}
                                    onChange={(e) =>
                                        setFormData((p) => ({ ...p, slippage: e.target.value }))
                                    }
                                />
                            </label>

                            <PriceCurveEditor curve={priceCurve} onChange={setPriceCurve} />
                        </div>
                    )}

                    {/* info + submit */}
                    <div className="swap-info">
                        <div className="info-row">
                            1 {formData.fromAsset.symbol} ≈{' '}
                            {formData.fromAmount && formData.toAmount
                                ? (
                                    Number(formData.toAmount) / Number(formData.fromAmount)
                                ).toFixed(6)
                                : '0'}{' '}
                            {formData.toAsset.symbol}
                            <span className="fee-info">
                                <Info size={14} /> Free
                            </span>
                        </div>
                    </div>

                    <button
                        type="submit"
                        className={`submit-button ${isFormValid ? 'enabled' : 'disabled'
                            }`}
                        disabled={!isFormValid || loading}
                    >
                        {loading
                            ? 'Creating…'
                            : !hasSourceWallet
                                ? `Connect ${fromChain} Wallet`
                                : 'Create Swap Order'}
                    </button>
                </form>
            </div>
        </div>
    );
};

export default NewOrder;
