import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect, useState, type FormEvent } from "react";
import { supabase } from "@/integrations/supabase/client";
import { useAuth } from "@/context/AuthContext";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { Plus, Trash2 } from "lucide-react";

export const Route = createFileRoute("/_authenticated/account")({
  head: () => ({
    meta: [
      { title: "Your account — Maison Auré" },
      { name: "description", content: "Orders, addresses, and account settings." },
    ],
  }),
  component: AccountPage,
});

type Tab = "orders" | "addresses" | "payment" | "profile";

function AccountPage() {
  const { user, signOut } = useAuth();
  const nav = useNavigate();
  const [tab, setTab] = useState<Tab>("orders");

  return (
    <>
      <Header />
      <main className="container-edge py-16 md:py-24 min-h-[70vh]">
        <div className="flex items-center gap-4 mb-4">
          <span className="gold-divider" />
          <span className="spec text-[10px]">Private — {user?.email}</span>
        </div>
        <div className="flex flex-wrap items-end justify-between gap-4 mb-12">
          <h1 className="font-serif text-5xl md:text-6xl tracking-[-0.02em]">Your account.</h1>
          <button
            onClick={async () => {
              await signOut();
              nav({ to: "/" });
            }}
            className="link-underline spec text-[10px]"
          >
            Sign out
          </button>
        </div>

        <div className="grid md:grid-cols-[220px_1fr] gap-10 md:gap-16">
          <aside className="md:border-r md:border-border md:pr-8">
            <nav className="flex md:flex-col gap-1 overflow-x-auto">
              {[
                ["orders", "Orders"],
                ["addresses", "Addresses"],
                ["payment", "Payment"],
                ["profile", "Profile"],
              ].map(([key, label]) => (
                <button
                  key={key}
                  onClick={() => setTab(key as Tab)}
                  className={`text-left spec text-[10px] py-2 px-3 border-l-2 transition-colors whitespace-nowrap ${
                    tab === key
                      ? "border-[var(--gold)] text-[var(--ink)]"
                      : "border-transparent text-muted-foreground hover:text-[var(--ink)]"
                  }`}
                >
                  {label}
                </button>
              ))}
            </nav>
          </aside>

          <section>
            {tab === "orders" && <OrdersPanel />}
            {tab === "addresses" && <AddressesPanel />}
            {tab === "payment" && <PaymentPanel />}
            {tab === "profile" && <ProfilePanel />}
          </section>
        </div>
      </main>
      <Footer />
    </>
  );
}

// ------------------------------ Orders ------------------------------

interface Order {
  id: string;
  order_number: string;
  status: string;
  total: number;
  created_at: string;
}

function OrdersPanel() {
  const [orders, setOrders] = useState<Order[] | null>(null);

  useEffect(() => {
    supabase
      .from("orders")
      .select("id, order_number, status, total, created_at")
      .order("created_at", { ascending: false })
      .then(({ data }) => setOrders(data ?? []));
  }, []);

  return (
    <PanelWrap title="Order history" caption="Every commission, archived.">
      {orders === null ? (
        <Loading />
      ) : orders.length === 0 ? (
        <Empty text="No orders yet. When you place one, it'll appear here." />
      ) : (
        <ul className="divide-y divide-border border-y border-border">
          {orders.map((o) => (
            <li key={o.id} className="py-5 flex items-center justify-between gap-4">
              <div>
                <div className="font-serif text-xl">#{o.order_number}</div>
                <div className="spec text-[9px] text-muted-foreground mt-1">
                  {new Date(o.created_at).toLocaleDateString()} · {o.status}
                </div>
              </div>
              <div className="font-mono text-sm tabular-nums text-[var(--gold)]">
                ${Number(o.total).toLocaleString()}
              </div>
            </li>
          ))}
        </ul>
      )}
    </PanelWrap>
  );
}

// ------------------------------ Addresses ------------------------------

interface Address {
  id: string;
  label: string | null;
  recipient: string;
  street: string;
  city: string;
  postal_code: string;
  country: string;
  phone: string | null;
  is_default: boolean;
}

function AddressesPanel() {
  const [list, setList] = useState<Address[] | null>(null);
  const [adding, setAdding] = useState(false);

  const load = async () => {
    const { data } = await supabase
      .from("addresses")
      .select("*")
      .order("created_at", { ascending: false });
    setList(data ?? []);
  };

  useEffect(() => {
    load();
  }, []);

  const remove = async (id: string) => {
    await supabase.from("addresses").delete().eq("id", id);
    load();
  };

  return (
    <PanelWrap
      title="Saved addresses"
      caption="For shipping and billing."
      action={
        !adding && (
          <button onClick={() => setAdding(true)} className="link-underline spec text-[10px] inline-flex items-center gap-2">
            <Plus className="w-3 h-3" /> Add address
          </button>
        )
      }
    >
      {adding && (
        <AddressForm
          onCancel={() => setAdding(false)}
          onSaved={() => {
            setAdding(false);
            load();
          }}
        />
      )}
      {list === null ? (
        <Loading />
      ) : list.length === 0 && !adding ? (
        <Empty text="No saved addresses." />
      ) : (
        <ul className="grid sm:grid-cols-2 gap-4 mt-4">
          {list.map((a) => (
            <li key={a.id} className="border border-border p-5 relative">
              {a.is_default && (
                <span className="absolute top-3 right-3 spec text-[8px] text-[var(--gold)]">Default</span>
              )}
              <div className="spec text-[9px] text-muted-foreground mb-2">{a.label ?? "Address"}</div>
              <div className="font-serif text-lg">{a.recipient}</div>
              <p className="text-sm text-muted-foreground mt-1 leading-relaxed">
                {a.street}<br />
                {a.city}, {a.postal_code}<br />
                {a.country}
                {a.phone && (<><br />{a.phone}</>)}
              </p>
              <button
                onClick={() => remove(a.id)}
                className="mt-4 inline-flex items-center gap-1.5 spec text-[9px] text-muted-foreground hover:text-[var(--ink)]"
              >
                <Trash2 className="w-3 h-3" /> Remove
              </button>
            </li>
          ))}
        </ul>
      )}
    </PanelWrap>
  );
}

function AddressForm({ onCancel, onSaved }: { onCancel: () => void; onSaved: () => void }) {
  const { user } = useAuth();
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [form, setForm] = useState({
    label: "Home",
    recipient: "",
    street: "",
    city: "",
    postal_code: "",
    country: "",
    phone: "",
    is_default: false,
  });

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    if (!user) return;
    setBusy(true);
    setErr(null);
    const { error } = await supabase.from("addresses").insert({ ...form, user_id: user.id });
    setBusy(false);
    if (error) return setErr(error.message);
    onSaved();
  };

  const set = (k: keyof typeof form) => (e: React.ChangeEvent<HTMLInputElement>) =>
    setForm({ ...form, [k]: e.target.value });

  return (
    <form onSubmit={submit} className="border border-border p-5 mt-4 grid sm:grid-cols-2 gap-4">
      <Field label="Label"><input className="auth-input" value={form.label} onChange={set("label")} /></Field>
      <Field label="Recipient"><input required className="auth-input" value={form.recipient} onChange={set("recipient")} /></Field>
      <Field label="Street" full><input required className="auth-input" value={form.street} onChange={set("street")} /></Field>
      <Field label="City"><input required className="auth-input" value={form.city} onChange={set("city")} /></Field>
      <Field label="Postal code"><input required className="auth-input" value={form.postal_code} onChange={set("postal_code")} /></Field>
      <Field label="Country"><input required className="auth-input" value={form.country} onChange={set("country")} /></Field>
      <Field label="Phone"><input className="auth-input" value={form.phone} onChange={set("phone")} /></Field>
      <label className="sm:col-span-2 inline-flex items-center gap-2 text-[12px]">
        <input
          type="checkbox"
          checked={form.is_default}
          onChange={(e) => setForm({ ...form, is_default: e.target.checked })}
        />
        Use as default address
      </label>
      {err && <p className="sm:col-span-2 text-sm text-[oklch(0.55_0.22_27)]">{err}</p>}
      <div className="sm:col-span-2 flex items-center gap-3">
        <button type="submit" disabled={busy} className="btn-solid">
          <span>{busy ? "Saving…" : "Save address"}</span>
        </button>
        <button type="button" onClick={onCancel} className="link-underline spec text-[10px]">
          Cancel
        </button>
      </div>
    </form>
  );
}

// ------------------------------ Payment ------------------------------

function PaymentPanel() {
  return (
    <PanelWrap title="Saved payment methods" caption="Cards and digital wallets.">
      <div className="border border-border p-8 text-center">
        <div className="spec text-[9px] text-[var(--gold)] mb-3">Coming with checkout</div>
        <p className="font-serif text-2xl mb-2">No saved methods yet.</p>
        <p className="text-sm text-muted-foreground max-w-md mx-auto">
          Payment cards will be securely stored via Stripe once checkout is enabled. We never store
          card numbers on our servers — only a tokenised reference.
        </p>
      </div>
    </PanelWrap>
  );
}

// ------------------------------ Profile ------------------------------

function ProfilePanel() {
  const { user } = useAuth();
  const [name, setName] = useState("");
  const [email, setEmail] = useState(user?.email ?? "");
  const [pw, setPw] = useState("");
  const [pw2, setPw2] = useState("");
  const [msg, setMsg] = useState<string | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!user) return;
    supabase
      .from("profiles")
      .select("display_name")
      .eq("id", user.id)
      .maybeSingle()
      .then(({ data }) => setName(data?.display_name ?? ""));
  }, [user]);

  const saveProfile = async (e: FormEvent) => {
    e.preventDefault();
    if (!user) return;
    setBusy(true);
    setMsg(null);
    setErr(null);
    const updates: { display_name: string } = { display_name: name };
    const { error: pErr } = await supabase.from("profiles").update(updates).eq("id", user.id);
    let emailErr: string | null = null;
    if (email && email !== user.email) {
      const { error } = await supabase.auth.updateUser({ email });
      if (error) emailErr = error.message;
    }
    setBusy(false);
    if (pErr || emailErr) return setErr(pErr?.message ?? emailErr ?? "Update failed");
    setMsg(emailErr ? "Saved." : "Saved. Check your inbox to confirm any email change.");
  };

  const savePassword = async (e: FormEvent) => {
    e.preventDefault();
    setMsg(null);
    setErr(null);
    if (pw.length < 6) return setErr("Password must be at least 6 characters.");
    if (pw !== pw2) return setErr("Passwords don't match.");
    setBusy(true);
    const { error } = await supabase.auth.updateUser({ password: pw });
    setBusy(false);
    if (error) return setErr(error.message);
    setPw("");
    setPw2("");
    setMsg("Password updated.");
  };

  return (
    <PanelWrap title="Profile settings" caption="Name, email, and password.">
      <form onSubmit={saveProfile} className="border border-border p-5 grid sm:grid-cols-2 gap-4">
        <Field label="Full name"><input required className="auth-input" value={name} onChange={(e) => setName(e.target.value)} /></Field>
        <Field label="Email"><input type="email" required className="auth-input" value={email} onChange={(e) => setEmail(e.target.value)} /></Field>
        <div className="sm:col-span-2">
          <button type="submit" disabled={busy} className="btn-solid">
            <span>{busy ? "Saving…" : "Save profile"}</span>
          </button>
        </div>
      </form>

      <form onSubmit={savePassword} className="border border-border p-5 mt-6 grid sm:grid-cols-2 gap-4">
        <div className="sm:col-span-2 spec text-[10px] text-muted-foreground">Change password</div>
        <Field label="New password"><input type="password" minLength={6} className="auth-input" value={pw} onChange={(e) => setPw(e.target.value)} /></Field>
        <Field label="Confirm"><input type="password" minLength={6} className="auth-input" value={pw2} onChange={(e) => setPw2(e.target.value)} /></Field>
        <div className="sm:col-span-2">
          <button type="submit" disabled={busy || !pw} className="btn-ghost">
            <span>{busy ? "Saving…" : "Update password"}</span>
          </button>
        </div>
      </form>

      {msg && <p className="mt-4 text-sm text-[var(--gold)]">{msg}</p>}
      {err && <p className="mt-4 text-sm text-[oklch(0.55_0.22_27)]">{err}</p>}
    </PanelWrap>
  );
}

// ------------------------------ Shared ------------------------------

function PanelWrap({
  title,
  caption,
  children,
  action,
}: {
  title: string;
  caption: string;
  children: React.ReactNode;
  action?: React.ReactNode;
}) {
  return (
    <div>
      <div className="flex items-end justify-between gap-4 mb-6">
        <div>
          <h2 className="font-serif text-3xl tracking-[-0.01em]">{title}</h2>
          <p className="spec text-[9px] text-muted-foreground mt-2">{caption}</p>
        </div>
        {action}
      </div>
      {children}
    </div>
  );
}

function Field({ label, children, full }: { label: string; children: React.ReactNode; full?: boolean }) {
  return (
    <label className={`block ${full ? "sm:col-span-2" : ""}`}>
      <span className="spec text-[10px] text-muted-foreground block mb-2">{label}</span>
      {children}
    </label>
  );
}

function Loading() {
  return <p className="spec text-[10px] text-muted-foreground">Loading…</p>;
}

function Empty({ text }: { text: string }) {
  return (
    <div className="border border-border p-8 text-center">
      <p className="text-sm text-muted-foreground">{text}</p>
    </div>
  );
}
