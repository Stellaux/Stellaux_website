import { createFileRoute, Link, useNavigate, redirect } from "@tanstack/react-router";
import { useState, type FormEvent } from "react";
import { z } from "zod";
import { supabase } from "@/integrations/supabase/client";
import { lovable } from "@/integrations/lovable/index";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";

const searchSchema = z.object({ redirect: z.string().optional() });

export const Route = createFileRoute("/login")({
  validateSearch: (s) => searchSchema.parse(s),
  head: () => ({
    meta: [
      { title: "Sign in — Maison Auré" },
      { name: "description", content: "Sign in or create your Maison Auré account." },
    ],
  }),
  component: LoginPage,
});

function LoginPage() {
  const nav = useNavigate();
  const search = Route.useSearch();
  const [mode, setMode] = useState<"signin" | "signup">("signin");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const goNext = () => nav({ to: search.redirect ?? "/account" });

  const handleEmail = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      if (mode === "signup") {
        const { error } = await supabase.auth.signUp({
          email,
          password,
          options: {
            emailRedirectTo: window.location.origin,
            data: { full_name: name },
          },
        });
        if (error) throw error;
        goNext();
      } else {
        const { error } = await supabase.auth.signInWithPassword({ email, password });
        if (error) throw error;
        goNext();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Authentication failed");
    } finally {
      setBusy(false);
    }
  };

  const handleGoogle = async () => {
    setError(null);
    setBusy(true);
    const result = await lovable.auth.signInWithOAuth("google", {
      redirect_uri: window.location.origin + "/login",
    });
    if (result.error) {
      setError(result.error.message ?? "Google sign-in failed");
      setBusy(false);
      return;
    }
    if (result.redirected) return;
    goNext();
  };

  return (
    <>
      <Header />
      <main className="container-edge py-20 md:py-28 min-h-[70vh]">
        <div className="max-w-md mx-auto">
          <div className="flex items-center gap-4 mb-6">
            <span className="gold-divider" />
            <span className="spec text-[10px]">{mode === "signin" ? "Returning Client" : "New Client"}</span>
          </div>
          <h1 className="font-serif text-5xl tracking-[-0.02em]">
            {mode === "signin" ? "Welcome back." : "Open an account."}
          </h1>
          <p className="mt-4 text-muted-foreground text-[14px]">
            {mode === "signin"
              ? "Access your bag, orders, and saved addresses."
              : "Save your details and follow each piece from atelier to door."}
          </p>

          <button
            onClick={handleGoogle}
            disabled={busy}
            className="mt-10 w-full inline-flex items-center justify-center gap-3 border border-border py-3 hover:border-[var(--ink)] transition-colors text-[13px]"
          >
            <GoogleIcon />
            Continue with Google
          </button>

          <div className="my-8 flex items-center gap-4">
            <div className="hairline flex-1" />
            <span className="spec text-[9px] text-muted-foreground">or</span>
            <div className="hairline flex-1" />
          </div>

          <form onSubmit={handleEmail} className="space-y-4">
            {mode === "signup" && (
              <Field label="Full name">
                <input
                  required
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="auth-input"
                  autoComplete="name"
                />
              </Field>
            )}
            <Field label="Email">
              <input
                type="email"
                required
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="auth-input"
                autoComplete="email"
              />
            </Field>
            <Field label="Password">
              <input
                type="password"
                required
                minLength={6}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="auth-input"
                autoComplete={mode === "signin" ? "current-password" : "new-password"}
              />
            </Field>

            {error && <p className="text-sm text-[oklch(0.55_0.22_27)]">{error}</p>}

            <button type="submit" disabled={busy} className="btn-solid w-full justify-center">
              <span>{busy ? "…" : mode === "signin" ? "Sign in" : "Create account"}</span>
            </button>
          </form>

          <div className="mt-6 flex items-center justify-between text-[12px]">
            <button
              onClick={() => setMode(mode === "signin" ? "signup" : "signin")}
              className="link-underline spec text-[10px]"
            >
              {mode === "signin" ? "Create an account" : "I already have an account"}
            </button>
            {mode === "signin" && (
              <Link to="/forgot-password" className="link-underline spec text-[10px]">
                Forgot password
              </Link>
            )}
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="block">
      <span className="spec text-[10px] text-muted-foreground block mb-2">{label}</span>
      {children}
    </label>
  );
}

function GoogleIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 48 48" aria-hidden>
      <path fill="#FFC107" d="M43.6 20.5H42V20H24v8h11.3c-1.6 4.6-6 8-11.3 8-6.6 0-12-5.4-12-12s5.4-12 12-12c3.1 0 5.9 1.2 8 3l5.7-5.7C34 6.1 29.3 4 24 4 12.9 4 4 12.9 4 24s8.9 20 20 20 20-8.9 20-20c0-1.3-.1-2.4-.4-3.5z" />
      <path fill="#FF3D00" d="M6.3 14.7l6.6 4.8C14.6 16 19 13 24 13c3.1 0 5.9 1.2 8 3l5.7-5.7C34 6.1 29.3 4 24 4 16.3 4 9.7 8.3 6.3 14.7z" />
      <path fill="#4CAF50" d="M24 44c5.2 0 9.9-2 13.4-5.2l-6.2-5.2c-2 1.5-4.6 2.4-7.2 2.4-5.3 0-9.7-3.4-11.3-8L6.2 32.6C9.5 39.2 16.2 44 24 44z" />
      <path fill="#1976D2" d="M43.6 20.5H42V20H24v8h11.3c-.8 2.3-2.2 4.3-4.1 5.6l6.2 5.2C40.6 36.9 44 31 44 24c0-1.3-.1-2.4-.4-3.5z" />
    </svg>
  );
}
