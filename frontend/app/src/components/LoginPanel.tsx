import React, { useEffect, useState } from 'react';
import icpLogo from '../assets/icp-logo.svg';
import ethereumLogo from '../assets/ethereum-logo.svg';
import { useIcpIdentity } from '../hooks/useIcpIdentity';
import { useDynamicContext } from '@dynamic-labs/sdk-react-core';

/* ---------- Domain types ---------- */
export type LoginAddress =
    | { ICP: { principal_id: string } }
    | { EVM: { address: string; chainId: number } };

interface Props {
    onSuccess: (addr: LoginAddress) => void;
}

/* ---------- Component ---------- */
const LoginPanel: React.FC<Props> = ({ onSuccess }) => {
    /* ICP state via custom hook */
    const {
        wallet: icpWallet,
        loading: loadingIcp,
        error: icpErr,
        connect: connectIcp,
    } = useIcpIdentity();

    /* Dynamic (EVM) */
    const {
        primaryWallet,
        setShowAuthFlow,        // opens Dynamic modal
        handleLogOut,           // disconnect
    } = useDynamicContext();

    const [evmLoading, setEvmLoading] = useState(false);
    const [evmError, setEvmError] = useState<string | null>(null);

    /* ----- Handlers ----- */

    // Internet Identity
    const handleIcpLogin = async () => {
        try {
            await connectIcp(false);
            if (icpWallet) {
                onSuccess({ ICP: { principal_id: icpWallet.principal.toText() } });
            }
        } catch (e: any) {
            console.error('ICP login error', e);
        }
    };

    // Dynamic EVM
    const handleEvmClick = async () => {
        setEvmError(null);
        if (primaryWallet) {
            // already connected → log out
            await handleLogOut();
            return;
        }
        setEvmLoading(true);
        try {
            setShowAuthFlow(true); // opens Dynamic modal
        } catch (e: any) {
            setEvmError(String(e));
        } finally {
            setEvmLoading(false);
        }
    };

    /* Fire onSuccess when Dynamic wallet becomes ready */
    useEffect(() => {
        if (primaryWallet) {
            onSuccess({
                EVM: { address: primaryWallet.address, chainId: Number(primaryWallet.chain) },
            });
        }
    }, [primaryWallet, onSuccess]);

    /* On mount: silent ICP auto-login if session exists */
    useEffect(() => {
        connectIcp(true).catch(() => {/* silent */ });
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    /* ----- Render ----- */
    return (
        <div className="bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-white rounded-xl p-6 max-w-lg mx-auto space-y-4">
            <h2 className="text-center text-2xl font-semibold mb-4">Sign in to Fusion+</h2>

            {/* ICP button */}
            <button
                className={`flex items-center justify-between w-full p-3 bg-gray-200 dark:bg-gray-700 rounded-md ${loadingIcp ? 'opacity-60 cursor-not-allowed' : 'hover:bg-gray-300 dark:hover:bg-gray-600'
                    }`}
                onClick={handleIcpLogin}
                disabled={loadingIcp}
            >
                <div className="flex items-center gap-3">
                    <img src={icpLogo} alt="ICP" className="h-6 w-6" />
                    {loadingIcp ? 'Signing in...' : 'Sign in with Internet Identity'}
                </div>
                {icpWallet && (
                    <span className="text-sm">
                        {icpWallet.principal.toText().slice(0, 6)}…
                        {icpWallet.principal.toText().slice(-4)}
                    </span>
                )}
            </button>
            {icpErr && <p className="text-sm text-red-500 break-all">II error: {icpErr}</p>}

            {/* EVM button */}
            <button
                className={`flex items-center justify-between w-full p-3 bg-gray-200 dark:bg-gray-700 rounded-md ${evmLoading ? 'opacity-60 cursor-not-allowed' : 'hover:bg-gray-300 dark:hover:bg-gray-600'
                    }`}
                onClick={handleEvmClick}
                disabled={evmLoading}
            >
                <div className="flex items-center gap-3">
                    <img src={ethereumLogo} alt="ETH" className="h-6 w-6" />
                    {primaryWallet ? 'Disconnect EVM Wallet' : 'Connect EVM Wallet'}
                </div>
                {primaryWallet && (
                    <span className="text-sm">
                        {primaryWallet.address.slice(0, 6)}…{primaryWallet.address.slice(-4)}
                    </span>
                )}
            </button>
            {evmError && <p className="text-sm text-red-500 break-all">{evmError}</p>}
        </div>
    );
};

export default LoginPanel;
