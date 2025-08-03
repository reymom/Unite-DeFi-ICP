import { useCallback, useState } from "react";
import { AuthClient } from "@dfinity/auth-client";
import { HttpAgent, Identity } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { icpHost, iiUrl } from "../model/icp";

export interface IcpWallet {
  principal: Principal;
  agent: HttpAgent;
}

export function useIcpIdentity() {
  const [wallet, setWallet] = useState<IcpWallet | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const connect = useCallback(async (silent = false): Promise<IcpWallet> => {
    setError(null);
    setLoading(true);
    try {
      const authClient = await AuthClient.create();
      if (!silent || !(await authClient.isAuthenticated())) {
        await authClient.login({
          identityProvider: iiUrl,
        });
      }

      const identity = authClient.getIdentity();
      const principal = identity.getPrincipal();
      const agent = await HttpAgent.create({
        identity: identity as unknown as Identity,
        host: icpHost,
      });
      if (process.env.VITE_ICP_ENV === "test") {
        agent.fetchRootKey();
      }

      const icpWallet = { principal, agent } as unknown as IcpWallet;
      setWallet(icpWallet);
      return icpWallet;
    } catch (e: any) {
      setError(String(e));
      throw e;
    } finally {
      setLoading(false);
    }
  }, []);

  const disconnect = useCallback(async () => {
    const authClient = await AuthClient.create();
    await authClient.logout();
    setWallet(null);
  }, []);

  return { wallet, loading, error, connect, disconnect };
}
