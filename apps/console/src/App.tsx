import { useState } from "react";
import { getStoredToken, setStoredToken } from "./api/local-storage";
import { SignInPage } from "./pages/sign-in/SignInPage";
import { DashboardPage } from "./pages/dashboard/DashboardPage";

export function App() {
  const [authenticated, setAuthenticated] = useState<boolean>(() => Boolean(getStoredToken()));

  if (!authenticated) {
    return <SignInPage onSuccess={() => setAuthenticated(true)} />;
  }

  return (
    <DashboardPage
      onSignOut={() => {
        setStoredToken(null);
        setAuthenticated(false);
      }}
    />
  );
}
