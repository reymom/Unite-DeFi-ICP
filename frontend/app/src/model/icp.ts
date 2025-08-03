export const icpHost =
  process.env.VITE_ICP_ENV === "test"
    ? "http://127.0.0.1:8080"
    : "https://ic0.app";

export const iiUrl =
  process.env.VITE_ICP_ENV === "production"
    ? `https://identity.ic0.app`
    : `http://${process.env.CANISTER_ID_INTERNET_IDENTITY}.localhost:8080`;
