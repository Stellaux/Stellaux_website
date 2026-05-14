import { createFileRoute, Link } from "@tanstack/react-router";
import { useState, type FormEvent } from "react";
import { supabase } from "@/integrations/supabase/client";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";

export const Route = createFileRoute("/forgot-password")({
  head: () => ({
    meta: [
      { title: "Forgot password — Maison Auré" },
      { name: "description", content: "Reset your Maison Auré account password." },
    ],
  }),
  component: ForgotPage,
});

function ForgotPage() {
  const [email, setEmail] = useState("");
  const [sent, setSent] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    setBusy(true);
    const { error } = await supabase.auth.resetPasswordForEmail(email, {
      redirectTo: window.location.origin + "/reset-password",
    });
    setBusy(false);
    if (error) {
      setError(error.message);
      return;
    }
    setSent(true);
  };

  return (
    <>
      <Header />
      <main className="container-edge py-20 md:py-28 min-h-[70vh]">
        <div className="max-w-md mx-auto">
          <div className="flex items-center gap-4 mb-6">
            <span className="gold-divider" />
            <span className="spec text-[10px]">Reset</span>
          </div>
          <h1 className="font-serif text-5xl tracking-[-0.02em]">Forgot password.</h1>
          <p className="mt-4 text-muted-foreground text-[14px]">
            Enter the email associated with your account. We'll send a secure link to set a new password.
          </p>

          {sent ? (
            <div className="mt-10 border border-border p-6">
              <p className="font-serif text-2xl">Check your inbox.</p>
              <p className="mt-2 text-sm text-muted-foreground">
                If an account exists for <strong>{email}</strong>, a reset link is on its way.
              </p>
            </div>
          ) : (
            <form onSubmit={submit} className="mt-10 space-y-4">
              <label className="block">
                <span className="spec text-[10px] text-muted-foreground block mb-2">Email</span>
                <input
                  type="email"
                  required
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="auth-input"
                  autoComplete="email"
                />
              </label>
              {error && <p className="text-sm text-[oklch(0.55_0.22_27)]">{error}</p>}
              <button type="submit" disabled={busy} className="btn-solid w-full justify-center">
                <span>{busy ? "Sending…" : "Send reset link"}</span>
              </button>
            </form>
          )}

          <div className="mt-6">
            <Link to="/login" className="link-underline spec text-[10px]">
              Back to sign in
            </Link>
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}
