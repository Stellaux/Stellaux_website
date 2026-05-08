import { createFileRoute, Link } from "@tanstack/react-router";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { useCart } from "@/context/CartContext";

export const Route = createFileRoute("/checkout")({
  head: () => ({
    meta: [
      { title: "Checkout — Maison Auré" },
      { name: "description", content: "Secure checkout — sign in or continue as guest." },
    ],
  }),
  component: CheckoutPage,
});

function CheckoutPage() {
  const { items, subtotal } = useCart();
  return (
    <>
      <Header />
      <main className="container-edge py-20 md:py-28">
        <div className="max-w-2xl">
          <div className="flex items-center gap-4 mb-6">
            <span className="gold-divider" />
            <span className="spec text-[10px]">Checkout · Step 0 of 3</span>
          </div>
          <h1 className="font-serif text-5xl md:text-6xl tracking-[-0.02em]">
            A moment, please.
          </h1>
          <p className="mt-6 text-muted-foreground text-[15px] leading-relaxed max-w-lg">
            Authentication, shipping, and Stripe payment will be activated when Lovable Cloud and
            payments are enabled on this project. Your bag is preserved and ready.
          </p>

          <div className="mt-12 border border-border p-6 md:p-8">
            <div className="spec text-[10px] mb-4">Your bag · {items.length} pieces</div>
            {items.length === 0 ? (
              <p className="text-sm text-muted-foreground">Nothing in the bag yet.</p>
            ) : (
              <ul className="space-y-3">
                {items.map((i) => (
                  <li key={i.id} className="flex items-baseline justify-between text-sm">
                    <span className="font-serif text-lg">
                      {i.name} <span className="spec text-[9px] text-muted-foreground">× {i.qty}</span>
                    </span>
                    <span className="font-mono tabular-nums">${(i.price * i.qty).toLocaleString()}</span>
                  </li>
                ))}
              </ul>
            )}
            <div className="hairline my-5" />
            <div className="flex items-baseline justify-between">
              <span className="spec text-[10px]">Subtotal</span>
              <span className="font-mono text-base text-[var(--gold)] tabular-nums">
                ${subtotal.toLocaleString()}
              </span>
            </div>
          </div>

          <div className="mt-10 flex items-center gap-4">
            <Link to="/shop" className="btn-ghost">
              <span>Back to the collection</span>
            </Link>
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}
