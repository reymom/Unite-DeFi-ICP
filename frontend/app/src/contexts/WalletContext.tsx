import React, { createContext, useContext } from 'react';
import { IcpWallet, useIcpIdentity } from '../hooks/useIcpIdentity';
import { useDynamicContext } from '@dynamic-labs/sdk-react-core';

export interface WalletContextShape {
    /** ICP */
    icp: {
        wallet: ReturnType<typeof useIcpIdentity>['wallet'];
        loading: boolean;
        error: string | null;
        connect: () => Promise<IcpWallet>;
        disconnect: () => Promise<void>;
    };
    /** EVM */
    evm: {
        address: string | null;
        chainId: number | null;
        connected: boolean;
        connect: () => Promise<void>;
        disconnect: () => Promise<void>;
    };
}

const WalletContext = createContext<WalletContextShape | undefined>(undefined);
export const useWallet = () => {
    const ctx = useContext(WalletContext);
    if (!ctx) throw new Error('useWallet must be used inside WalletProvider');
    return ctx;
};

export const WalletProvider: React.FC<{ children: React.ReactNode }> = ({
    children,
}) => {
    /* ICP hook */
    const icp = useIcpIdentity();

    /* EVM via Dynamic */
    const {
        user,
        setShowAuthFlow, // opens Dynamic modal
        handleLogOut,
        primaryWallet,
    } = useDynamicContext();

    const evmConnected = !!user && !!primaryWallet;
    const evmAddress = evmConnected ? primaryWallet.address : null;
    const evmChainId = evmConnected ? Number(primaryWallet.chain) : null;

    const connectEvm = async () => setShowAuthFlow(true);
    const disconnectEvm = async () => handleLogOut();

    return (
        <WalletContext.Provider
            value={{
                icp: {
                    wallet: icp.wallet,
                    loading: icp.loading,
                    error: icp.error,
                    connect: () => icp.connect(false),
                    disconnect: icp.disconnect,
                },
                evm: {
                    address: evmAddress,
                    chainId: evmChainId,
                    connected: evmConnected,
                    connect: connectEvm,
                    disconnect: disconnectEvm,
                },
            }}
        >
            {children}
        </WalletContext.Provider>
    );
};