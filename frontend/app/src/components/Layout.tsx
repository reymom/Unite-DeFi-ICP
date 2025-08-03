import type React from "react"
import { Link, useLocation } from "react-router-dom"
import { Settings, Home, ArrowLeftRight, HelpCircle } from "lucide-react"
import WalletConnector from "./WalletConnector"

interface LayoutProps {
    children: React.ReactNode
}

const Layout: React.FC<LayoutProps> = ({ children }) => {
    const location = useLocation()

    const navItems = [
        { path: "/", label: "Dashboard", icon: Home },
        { path: "/swap", label: "Swap", icon: ArrowLeftRight },
        { path: "/settings", label: "Settings", icon: Settings },
    ]

    return (
        <div className="layout">
            <header className="header">
                <div className="header-content">
                    <div className="logo-section">
                        <img src="/icp-fully-onchain.svg" alt="ICP" className="logo-icp" />
                        <h1 className="app-title">Fusion+ ICP Swap</h1>
                    </div>

                    <nav className="nav">
                        {navItems.map((item) => {
                            const Icon = item.icon
                            return (
                                <Link
                                    key={item.path}
                                    to={item.path}
                                    className={`nav-item ${location.pathname === item.path ? "active" : ""}`}
                                >
                                    <Icon size={18} />
                                    <span>{item.label}</span>
                                </Link>
                            )
                        })}
                    </nav>

                    <div className="header-actions">
                        <button className="icon-button">
                            <HelpCircle size={20} />
                        </button>
                        <WalletConnector />
                    </div>
                </div>
            </header>

            <main className="main-content">{children}</main>

            <footer className="footer">
                <div className="footer-content">
                    <div className="footer-logos">
                        <img src="/icp-fully-onchain.svg" alt="ICP Fully Onchain" className="footer-logo" />
                        <img src="/1inch_logo.png" alt="1inch" className="footer-logo" />
                    </div>
                    <div className="footer-text">
                        <p>Powered by Internet Computer & 1inch Fusion+</p>
                    </div>
                </div>
            </footer>
        </div>
    )
}

export default Layout
