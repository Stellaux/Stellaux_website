import { useEffect } from "react";
import { Link } from "@tanstack/react-router";
import { Minus, Plus, X } from "lucide-react";
import { useCart } from "@/context/CartContext";

const SHIPPING_THRESHOLD = 200;
const FLAT_SHIPPING = 18;
const TAX_RATE = 0.08;

export function CartDrawer() {
  const { items, isOpen, close, removeItem, setQty, subtotal } = useCart();

  useEffect(() => {
    if (!isOpen) return;
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && close();
    document.addEventListener("keydown", onKey);
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("keydown", onKey);
      document.body.style.overflow = "";
    };
  }, [isOpen, close]);

  const shipping = subtotal === 0 ? 0 : subtotal >= SHIPPING_THRESHOLD ? 0 : FLAT_SHIPPING;
  const tax = +(subtotal * TAX_RATE).toFixed(2);
  const total = +(subtotal + shipping + tax).toFixed(2);

  return (
    <div
      className={`fixed inset-0 z-[60] ${isOpen ? "pointer-events-auto" : "pointer-events-none"}`}
      aria-hidden={!isOpen}
    >
      <div
        onClick={close}
        className={`absolute inset-0 bg-black/40 transition-opacity duration-500 ${
          isOpen ? "opacity-100" : "opacity-0"
        }`}
      />
      <aside
        role="dialog"
        aria-label="Shopping cart"
        className={`absolute right-0 top-0 h-full w-full max-w-md bg-background shadow-[var(--shadow-elevated)] flex flex-col transition-transform duration-500 ease-[cubic-bezier(0.2,0.7,0.2,1)] ${
          isOpen ? "translate-x-0" : "translate-x-full"
        }`}
      >
        <header className="flex items-center justify-between px-6 py-5 border-b border-border">
          <div className="flex items-center gap-3">
            <span className="gold-divider" />
            <span className="spec text-[11px]">The Bag · {items.length}</span>
          </div>
          <button onClick={close} aria-label="Close cart" className="p-2 -mr-2 hover:text-[var(--gold)]">
            <X className="w-5 h-5" strokeWidth={1.4} />
          </button>
        </header>

        {items.length === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center px-6 text-center">
            <p className="font-serif text-3xl mb-2">Your bag is empty</p>
            <p className="spec text-[10px] text-muted-foreground mb-8">No pieces selected — yet.</p>
            <Link to="/shop" onClick={close} className="btn-ghost">
              <span>Browse the collection</span>
              <span aria-hidden>→</span>
            </Link>
          </div>
        ) : (
          <>
            <div className="flex-1 overflow-y-auto px-6 py-4 divide-y divide-border">
              {items.map((it) => (
                <div key={it.id} className="flex gap-4 py-5">
                  <Link
                    to="/shop/$slug"
                    params={{ slug: it.slug }}
                    onClick={close}
                    className="shrink-0 w-24 h-24 bg-[var(--paper)] overflow-hidden"
                  >
                    <img src={it.image} alt={it.name} className="w-full h-full object-cover" />
                  </Link>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0">
                        <Link
                          to="/shop/$slug"
                          params={{ slug: it.slug }}
                          onClick={close}
                          className="font-serif text-lg leading-tight hover:text-[var(--gold)] block truncate"
                        >
                          {it.name}
                        </Link>
                        <div className="spec text-[9px] text-muted-foreground mt-1 flex gap-2 flex-wrap">
                          <span>{it.material}</span>
                          {it.size && <span>· Size {it.size}</span>}
                          <span>· {it.spec}</span>
                        </div>
                      </div>
                      <button
                        onClick={() => removeItem(it.id)}
                        aria-label={`Remove ${it.name}`}
                        className="p-1 -m-1 text-muted-foreground hover:text-[var(--ink)]"
                      >
                        <X className="w-3.5 h-3.5" />
                      </button>
                    </div>

                    <div className="mt-3 flex items-center justify-between">
                      <div className="inline-flex items-center border border-border">
                        <button
                          onClick={() => setQty(it.id, it.qty - 1)}
                          aria-label="Decrease"
                          className="w-8 h-8 flex items-center justify-center hover:bg-[var(--paper)]"
                        >
                          <Minus className="w-3 h-3" />
                        </button>
                        <span className="w-8 text-center font-mono text-xs tabular-nums">{it.qty}</span>
                        <button
                          onClick={() => setQty(it.id, it.qty + 1)}
                          aria-label="Increase"
                          className="w-8 h-8 flex items-center justify-center hover:bg-[var(--paper)]"
                        >
                          <Plus className="w-3 h-3" />
                        </button>
                      </div>
                      <span className="font-mono text-sm tabular-nums text-[var(--gold)]">
                        ${(it.price * it.qty).toLocaleString()}
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>

            <footer className="border-t border-border px-6 py-5 space-y-4">
              <dl className="space-y-2 text-sm">
                <Row label="Subtotal" value={`$${subtotal.toLocaleString()}`} />
                <Row
                  label="Shipping (est.)"
                  value={shipping === 0 ? "Complimentary" : `$${shipping}`}
                />
                <Row label="Tax (est.)" value={`$${tax.toLocaleString()}`} />
                <div className="hairline" />
                <Row label="Total" value={`$${total.toLocaleString()}`} bold />
              </dl>
              <p className="spec text-[9px] text-muted-foreground">
                Final taxes &amp; shipping calculated after address.
              </p>
              <Link to="/checkout" onClick={close} className="btn-solid w-full justify-center">
                <span>Proceed to checkout</span>
                <span aria-hidden>→</span>
              </Link>
              <button onClick={close} className="block w-full link-underline spec text-[10px] text-center">
                Continue shopping
              </button>
            </footer>
          </>
        )}
      </aside>
    </div>
  );
}

function Row({ label, value, bold }: { label: string; value: string; bold?: boolean }) {
  return (
    <div className="flex items-baseline justify-between">
      <dt className="spec text-[10px] text-muted-foreground">{label}</dt>
      <dd className={`font-mono tabular-nums ${bold ? "text-base text-[var(--gold)]" : "text-sm"}`}>{value}</dd>
    </div>
  );
}
