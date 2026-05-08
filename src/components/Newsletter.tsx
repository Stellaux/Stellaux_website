import { useState } from "react";

export function Newsletter() {
  const [email, setEmail] = useState("");
  const [sent, setSent] = useState(false);

  return (
    <section className="py-24 md:py-32 bg-background">
      <div className="container-edge">
        <div className="grid grid-cols-1 md:grid-cols-12 gap-10 items-end">
          <div className="md:col-span-6">
            <div className="flex items-center gap-4 mb-6">
              <span className="gold-divider" />
              <span className="spec text-[10px]">The Dispatch · Quarterly</span>
            </div>
            <h2 className="font-serif text-4xl md:text-5xl tracking-[-0.02em]">
              Join the dispatch.
            </h2>
            <p className="mt-5 text-muted-foreground max-w-md text-[15px] leading-relaxed">
              Four letters a year — new editions, atelier notes, and considered objects.
              No noise.
            </p>
          </div>

          <form
            className="md:col-span-6"
            onSubmit={(e) => {
              e.preventDefault();
              if (email) setSent(true);
            }}
          >
            <label className="spec text-[10px] block mb-3" htmlFor="email">
              001 · Your email
            </label>
            <div className="flex border-b border-[var(--ink)]">
              <input
                id="email"
                type="email"
                required
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="you@studio.com"
                className="flex-1 bg-transparent py-3 text-base outline-none placeholder:text-muted-foreground"
              />
              <button type="submit" className="spec text-[11px] px-2 hover:text-[var(--gold)] transition-colors">
                Subscribe →
              </button>
            </div>
            <p className="spec text-[9px] mt-4 text-muted-foreground">
              {sent ? "✓ Received. Welcome to the dispatch." : "We respect your inbox. Unsubscribe at any time."}
            </p>
          </form>
        </div>
      </div>
    </section>
  );
}
