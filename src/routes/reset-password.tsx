import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect, useState, type FormEvent } from "react";
import { supabase } from "@/integrations/supabase/client";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";

export const Route = createFileRoute("/reset-password")({
  head: () => ({
    meta: [
      { title: "Set new password — Maison Auré" },
      { name: "description", content: "Choose a new password for your account." },
    ],
  }),
  component: ResetPage,
});

function ResetPage() {
  const nav = useNavigate();
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    // Supabase auto-handles the recovery hash and fires onAuthStateChange.
    const { data: sub } = supabase.auth.onAuthStateChange((event) => {
      if (event === "PASSWORD_RECOVERY" || event === "SIGNED_IN") setReady(true);
    });
    supabase.auth.getSession().then(({ data }) => {
      if (data.session) setReady(true);
    });
    return () => sub.subscription.unsubscribe();
  }, []);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    if (password.length < 6) return setError("Password must be at least 6 characters.");
    if (password !== confirm) return setError("Passwords don't match.");
    setBusy(true);
    const { error } = await supabase.auth.updateUser({ password });
    setBusy(false);
    if (error) return setError(error.message);
    nav({ to: "/account" });
  };

  return (
    <>
      <Header />
      <main className="container-edge py-20 md:py-28 min-h-[70vh]">
        <div className="max-w-md mx-auto">
          <div className="flex items-center gap-4 mb-6">
            <span className="gold-divider" />
            <span className="spec text-[10px]">New password</span>
          </div>
          <h1 className="font-serif text-5xl tracking-[-0.02em]">Set a new password.</h1>
          <p className="mt-4 text-muted-foreground text-[14px]">
            Choose something memorable but uncommon. Six characters minimum.
          </p>

          {!ready ? (
            <p className="mt-10 spec text-[10px] text-muted-foreground">Verifying reset link…</p>
          ) : (
            <form onSubmit={submit} className="mt-10 space-y-4">
              <label className="block">
                <span className="spec text-[10px] text-muted-foreground block mb-2">New password</span>
                <input
                  type="password"
                  required
                  minLength={6}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="auth-input"
                  autoComplete="new-password"
                />
              </label>
              <label className="block">
                <span className="spec text-[10px] text-muted-foreground block mb-2">Confirm</span>
                <input
                  type="password"
                  required
                  minLength={6}
                  value={confirm}
                  onChange={(e) => setConfirm(e.target.value)}
                  className="auth-input"
                  autoComplete="new-password"
                />
              </label>
              {error && <p className="text-sm text-[oklch(0.55_0.22_27)]">{error}</p>}
              <button type="submit" disabled={busy} className="btn-solid w-full justify-center">
                <span>{busy ? "Saving…" : "Update password"}</span>
              </button>
            </form>
          )}
        </div>
      </main>
      <Footer />
    </>
  );
}
