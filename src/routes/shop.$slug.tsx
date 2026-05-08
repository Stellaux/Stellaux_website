import { createFileRoute, Link, notFound } from "@tanstack/react-router";
import { useState } from "react";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { ProductCard } from "@/components/ProductCard";
import { getProductBySlug, relatedProducts, type Product } from "@/data/products";
import { ChevronDown, ChevronLeft, ChevronRight, Heart, Minus, Plus } from "lucide-react";
import { useCart } from "@/context/CartContext";

const RING_SIZES = ["5", "6", "7", "8", "9", "10"];

export const Route = createFileRoute("/shop/$slug")({
  loader: ({ params }) => {
    const product = getProductBySlug(params.slug);
    if (!product) throw notFound();
    return { product };
  },
  head: ({ loaderData }) => {
    const p = loaderData?.product;
    if (!p) return {};
    return {
      meta: [
        { title: `${p.name} — Maison Auré` },
        { name: "description", content: p.description },
        { property: "og:title", content: `${p.name} — Maison Auré` },
        { property: "og:description", content: p.description },
        { property: "og:image", content: p.images[0] },
      ],
    };
  },
  notFoundComponent: () => (
    <>
      <Header />
      <main className="container-edge py-32 text-center">
        <h1 className="font-serif text-5xl">Piece not found</h1>
        <Link to="/shop" className="link-underline spec text-[11px] mt-6 inline-block">Back to the collection</Link>
      </main>
      <Footer />
    </>
  ),
  component: ProductPage,
});

function ProductPage() {
  const { product } = Route.useLoaderData();
  return (
    <>
      <Header />
      <main>
        <Crumbs product={product} />
        <ProductHero product={product} />
        <Related product={product} />
      </main>
      <Footer />
    </>
  );
}

function Crumbs({ product }: { product: Product }) {
  return (
    <div className="container-edge pt-8 pb-2">
      <nav className="spec text-[10px] text-muted-foreground flex items-center gap-2">
        <Link to="/" className="hover:text-[var(--ink)]">Home</Link>
        <span>/</span>
        <Link to="/shop" className="hover:text-[var(--ink)]">Shop</Link>
        <span>/</span>
        <span className="text-[var(--ink)] capitalize">{product.category}</span>
        <span>/</span>
        <span>No. {product.id}</span>
      </nav>
    </div>
  );
}

function ProductHero({ product }: { product: Product }) {
  const [active, setActive] = useState(0);
  const [size, setSize] = useState<string | null>(product.category === "rings" ? "7" : null);
  const [qty, setQty] = useState(1);
  const [zoom, setZoom] = useState({ on: false, x: 50, y: 50 });
  const { addItem } = useCart();

  const needsSize = product.category === "rings";
  const canAdd = !needsSize || !!size;

  const handleAdd = () => {
    if (!canAdd) return;
    addItem({
      productId: product.id,
      slug: product.slug,
      name: product.name,
      image: product.images[0],
      price: product.price,
      size,
      qty,
      spec: product.spec,
      material: product.material,
    });
  };

  const next = () => setActive((i) => (i + 1) % product.images.length);
  const prev = () => setActive((i) => (i - 1 + product.images.length) % product.images.length);

  return (
    <section className="container-edge py-8 md:py-12">
      <div className="grid grid-cols-1 md:grid-cols-12 gap-8 md:gap-14">
        {/* Gallery */}
        <div className="md:col-span-7">
          <div className="grid grid-cols-12 gap-3 md:gap-4">
            <div className="col-span-12 md:col-span-2 order-2 md:order-1">
              <div className="flex md:flex-col gap-3 overflow-x-auto md:overflow-visible">
                {product.images.map((src, i) => (
                  <button
                    key={i}
                    onClick={() => setActive(i)}
                    className={`shrink-0 w-20 md:w-full aspect-square overflow-hidden border transition-colors ${
                      active === i ? "border-[var(--ink)]" : "border-transparent hover:border-border"
                    }`}
                    aria-label={`View image ${i + 1}`}
                  >
                    <img src={src} alt="" className="w-full h-full object-cover" />
                  </button>
                ))}
              </div>
            </div>

            <div className="col-span-12 md:col-span-10 order-1 md:order-2">
              <div
                className="relative aspect-square bg-[var(--paper)] overflow-hidden cursor-zoom-in"
                onMouseEnter={() => setZoom((z) => ({ ...z, on: true }))}
                onMouseLeave={() => setZoom((z) => ({ ...z, on: false }))}
                onMouseMove={(e) => {
                  const r = e.currentTarget.getBoundingClientRect();
                  setZoom({ on: true, x: ((e.clientX - r.left) / r.width) * 100, y: ((e.clientY - r.top) / r.height) * 100 });
                }}
              >
                <img
                  src={product.images[active]}
                  alt={product.name}
                  width={1024}
                  height={1024}
                  className="w-full h-full object-cover transition-transform duration-500"
                  style={zoom.on ? { transform: "scale(1.6)", transformOrigin: `${zoom.x}% ${zoom.y}%` } : undefined}
                />

                <span className="absolute top-4 left-4 spec text-[10px] bg-white/90 px-2 py-1">
                  Fig. {String(active + 1).padStart(2, "0")} / {String(product.images.length).padStart(2, "0")}
                </span>
                <span className="absolute top-4 right-4 spec text-[10px] bg-white/90 px-2 py-1">
                  {product.spec}
                </span>

                <button onClick={prev} aria-label="Previous" className="absolute left-3 top-1/2 -translate-y-1/2 w-10 h-10 bg-white/90 hover:bg-[var(--ink)] hover:text-white flex items-center justify-center transition-colors">
                  <ChevronLeft className="w-4 h-4" />
                </button>
                <button onClick={next} aria-label="Next" className="absolute right-3 top-1/2 -translate-y-1/2 w-10 h-10 bg-white/90 hover:bg-[var(--ink)] hover:text-white flex items-center justify-center transition-colors">
                  <ChevronRight className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* Info */}
        <div className="md:col-span-5 md:pt-4">
          <div className="md:sticky md:top-28">
            <div className="flex items-center gap-4 mb-4">
              <span className="gold-divider" />
              <span className="spec text-[10px]">{product.collection} · No. {product.id}</span>
            </div>
            <h1 className="font-serif text-4xl md:text-5xl tracking-[-0.02em] leading-[1.05]">
              {product.name}
            </h1>
            <div className="mt-4 flex items-baseline gap-3">
              <span className="font-mono text-xl text-[var(--gold)] tabular-nums">${product.price}</span>
              <span className="spec text-[10px] text-muted-foreground">USD · Tax included EU</span>
            </div>

            <p className="mt-6 text-[15px] leading-relaxed text-muted-foreground">
              {product.description}
            </p>

            <div className="mt-6 grid grid-cols-3 gap-4 border-t border-border pt-5">
              <Spec k="Material" v={product.material} />
              <Spec k="Plating" v="06 μm" />
              <Spec k="Origin" v="IT" />
            </div>

            {/* Size */}
            {product.category === "rings" && (
              <div className="mt-8">
                <div className="flex items-center justify-between mb-3">
                  <span className="spec text-[10px]">Size · US</span>
                  <button className="spec text-[10px] text-muted-foreground link-underline">Size guide</button>
                </div>
                <div className="grid grid-cols-6 gap-2">
                  {RING_SIZES.map((s) => (
                    <button
                      key={s}
                      onClick={() => setSize(s)}
                      className={`py-3 spec text-[11px] border transition-colors ${
                        size === s
                          ? "border-[var(--ink)] bg-[var(--ink)] text-[var(--paper)]"
                          : "border-border hover:border-[var(--ink)]"
                      }`}
                    >
                      {s}
                    </button>
                  ))}
                </div>
              </div>
            )}

            {/* Quantity */}
            <div className="mt-8">
              <span className="spec text-[10px] block mb-3">Quantity</span>
              <div className="inline-flex items-center border border-[var(--ink)]">
                <button onClick={() => setQty((q) => Math.max(1, q - 1))} aria-label="Decrease" className="w-11 h-11 flex items-center justify-center hover:bg-[var(--paper)]">
                  <Minus className="w-3.5 h-3.5" />
                </button>
                <span className="w-10 text-center font-mono text-sm tabular-nums">{qty}</span>
                <button onClick={() => setQty((q) => q + 1)} aria-label="Increase" className="w-11 h-11 flex items-center justify-center hover:bg-[var(--paper)]">
                  <Plus className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>

            <div className="mt-8 flex items-stretch gap-3">
              <button
                onClick={handleAdd}
                disabled={!canAdd}
                className="btn-solid flex-1 justify-center disabled:opacity-50 disabled:cursor-not-allowed"
                style={{ borderColor: "var(--gold)" }}
              >
                <span>{canAdd ? `Add to cart · $${(product.price * qty).toLocaleString()}` : "Select a size"}</span>
              </button>
              <button aria-label="Save" className="border border-[var(--ink)] w-12 flex items-center justify-center hover:bg-[var(--ink)] hover:text-[var(--paper)] transition-colors">
                <Heart className="w-4 h-4" strokeWidth={1.4} />
              </button>
            </div>

            <div className="mt-5 spec text-[10px] text-muted-foreground flex flex-wrap gap-x-5 gap-y-1">
              <span>✦ Free worldwide shipping</span>
              <span>✦ 30-day returns</span>
              <span>✦ Lifetime polish</span>
            </div>

            {/* Accordions */}
            <div className="mt-10 border-t border-border">
              <Accordion title="Details & Care" defaultOpen>
                <ul className="space-y-2">
                  {product.details.map((d) => (
                    <li key={d} className="text-sm text-muted-foreground flex gap-2">
                      <span className="text-[var(--gold)]">—</span> {d}
                    </li>
                  ))}
                </ul>
                <p className="mt-4 text-sm text-muted-foreground">
                  Polish with the included microfiber. Avoid contact with perfume, chlorine, and saltwater.
                </p>
              </Accordion>
              <Accordion title="Shipping & Returns">
                <p className="text-sm text-muted-foreground leading-relaxed">
                  Complimentary express shipping worldwide on orders over $200. Each piece is shipped insured in a signature wood-and-paper case. Returns accepted within 30 days, in original condition.
                </p>
              </Accordion>
              <Accordion title="The Atelier">
                <p className="text-sm text-muted-foreground leading-relaxed">
                  Designed in our New York studio and hand-finished in Florence by a single team of master goldsmiths. Every piece is engraved with its dossier number and material grade.
                </p>
              </Accordion>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function Spec({ k, v }: { k: string; v: string }) {
  return (
    <div>
      <div className="spec text-[9px] text-muted-foreground">{k}</div>
      <div className="font-mono text-sm mt-1">{v}</div>
    </div>
  );
}

function Accordion({ title, children, defaultOpen = false }: { title: string; children: React.ReactNode; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className="border-b border-border">
      <button
        onClick={() => setOpen((v) => !v)}
        className="w-full py-5 flex items-center justify-between text-left group"
        aria-expanded={open}
      >
        <span className="spec text-[11px] group-hover:text-[var(--gold)] transition-colors">{title}</span>
        <ChevronDown className={`w-4 h-4 transition-transform ${open ? "rotate-180 text-[var(--gold)]" : ""}`} />
      </button>
      <div className={`grid transition-all duration-500 ${open ? "grid-rows-[1fr] pb-5" : "grid-rows-[0fr]"}`}>
        <div className="overflow-hidden">{children}</div>
      </div>
    </div>
  );
}

function Related({ product }: { product: Product }) {
  const items = relatedProducts(product);
  if (items.length === 0) return null;
  return (
    <section className="bg-[var(--paper)] py-24 md:py-32 mt-16 border-t border-border">
      <div className="container-edge">
        <div className="flex items-end justify-between mb-12">
          <div>
            <div className="flex items-center gap-4 mb-5">
              <span className="gold-divider" />
              <span className="spec text-[10px]">{product.collection} · The collection</span>
            </div>
            <h2 className="font-serif text-3xl md:text-5xl tracking-[-0.02em]">
              You may also consider
            </h2>
          </div>
          <Link to="/shop" className="hidden md:inline-flex link-underline spec text-[11px] pb-1">
            View all
          </Link>
        </div>
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-x-6 gap-y-14">
          {items.map((p, i) => (
            <ProductCard key={p.id} product={p} index={i} />
          ))}
        </div>
      </div>
    </section>
  );
}
