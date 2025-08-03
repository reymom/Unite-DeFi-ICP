import React from "react"
import { BrowserRouter as Router, Routes, Route } from "react-router-dom"
import Layout from "./components/Layout"
import Dashboard from "./pages/Dashboard"
import NewOrder from "./pages/NewOrder"
import TradeTimeline from "./pages/TradeTimeline"
import Settings from "./pages/Settings"
import { WalletProvider } from "./contexts/WalletContext"
import { OrderProvider } from "./contexts/OrderContext"

import { DynamicContextProvider } from '@dynamic-labs/sdk-react-core';
import { DynamicWagmiConnector } from '@dynamic-labs/wagmi-connector';

const App: React.FC = () => {
  return (
    <React.StrictMode>
      <DynamicContextProvider
        settings={{ environmentId: process.env.VITE_DYNAMIC_ENV_ID }}
      >
        <DynamicWagmiConnector>
          <WalletProvider>
            <OrderProvider>
              <Router>
                <div className="app">
                  <Layout>
                    <Routes>
                      <Route path="/" element={<Dashboard />} />
                      <Route path="/swap" element={<NewOrder />} />
                      <Route path="/trade/:orderId" element={<TradeTimeline />} />
                      <Route path="/settings" element={<Settings />} />
                    </Routes>
                  </Layout>
                </div>
              </Router>
            </OrderProvider>
          </WalletProvider>
        </DynamicWagmiConnector>
      </DynamicContextProvider>
    </React.StrictMode>
  )
}

export default App
