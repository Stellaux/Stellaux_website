import { createFileRoute, Outlet, useNavigate, Link } from "@tanstack/react-router";
import { useEffect } from "react";
import { useAuth } from "@/context/AuthContext";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";

export const Route = createFileRoute("/_authenticated")({
  component: AuthLayout,
});

function AuthLayout() {
  const { user, loading } = useAuth();
  const nav = useNavigate();

  useEffect(() => {
    if (!loading && !user) {
      nav({ to: "/login", search: { redirect: window.location.pathname } });
    }
  }, [loading, user, nav]);

  if (loading || !user) {
    return (
      <>
        <Header />
        <main className="container-edge py-32 min-h-[60vh]">
          <p className="spec text-[10px] text-muted-foreground">
            {loading ? "Verifying session…" : (
              <>Redirecting to <Link to="/login" className="link-underline">sign in</Link>…</>
            )}
          </p>
        </main>
        <Footer />
      </>
    );
  }

  return <Outlet />;
}
