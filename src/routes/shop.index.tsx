import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { fallback, zodValidator } from "@tanstack/zod-adapter";
import { useMemo, useState } from "react";
import { z } from "zod";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { ProductCard } from "@/components/ProductCard";
import { CATEGORIES, MATERIALS, products, type Category, type Material } from "@/data/products";
import { ChevronDown, SlidersHorizontal, X } from "lucide-react";

type SortKey = "newest" | "price-asc" | "price-desc" | "popularity";

const SORT_OPTIONS: { key: SortKey; label: string }[] = [
  { key: "newest", label: "Newest" },
  { key: "popularity", label: "Most popular" },
  { key: "price-asc", label: "Price · Low to high" },
  { key: "price-desc", label: "Price · High to low" },
];

const PRICE_RANGES = [
  { key: "all", label: "All prices", min: 0, max: Infinity },
  { key: "u250", label: "Under $250", min: 0, max: 250 },
  { key: "250-500", label: "$250 — $500", min: 250, max: 500 },
  { key: "500-1000", label: "$500 — $1,000", min: 500, max: 1000 },
  { key: "o1000", label: "Over $1,000", min: 1000, max: Infinity },
];

const PAGE_SIZE = 8;

const categoryEnum = z.enum(["all", "rings", "necklaces", "earrings", "bracelets"]);
const sortEnum = z.enum(["newest", "price-asc", "price-desc", "popularity"]);
const priceEnum = z.enum(["all", "u250", "250-500", "500-1000", "o1000"]);
const materialEnum = z.enum(["18k Gold", "Gold Vermeil", "Sterling Silver", "Platinum"]);

const shopSearchSchema = z.object({
  category: fallback(categoryEnum, "all").default("all"),
  sort: fallback(sortEnum, "newest").default("newest"),
  price: fallback(priceEnum, "all").default("all"),
  materials: fallback(z.array(materialEnum), []).default([]),
  page: fallback(z.number().int().min(1).max(20), 1).default(1),
});

export const Route = createFileRoute("/shop/")({
  validateSearch: zodValidator(shopSearchSchema),
  head: () => ({
    meta: [
      { title: "The Collection — Maison Auré" },
      { name: "description", content: "Browse rings, necklaces, earrings, and bracelets — engineered in 18K gold and platinum, hand-finished in Florence." },
      { property: "og:title", content: "The Collection — Maison Auré" },
      { property: "og:description", content: "Engineered jewelry for the modern professional." },
    ],
  }),
  component: ShopPage,
});

function ShopPage() {
  const search = Route.useSearch();
  const navigate = useNavigate({ from: "/shop/" });
  const [filtersOpen, setFiltersOpen] = useState(false);

  const { category, sort, price: priceKey, materials, page } = search;
  const visible = page * PAGE_SIZE;

  const update = (patch: Partial<typeof search>, resetPage = true) => {
    navigate({
      search: (prev: typeof search) => ({ ...prev, ...patch, ...(resetPage ? { page: 1 } : {}) }),
      replace: true,
    });
  };

  const filtered = useMemo(() => {
    const range = PRICE_RANGES.find((r) => r.key === priceKey)!;
    let list = products.filter((p) => {
      if (category !== "all" && p.category !== category) return false;
      if (materials.length && !materials.includes(p.material as Material)) return false;
      if (p.price < range.min || p.price > range.max) return false;
      return true;
    });
    list = [...list].sort((a, b) => {
      switch (sort) {
        case "price-asc": return a.price - b.price;
        case "price-desc": return b.price - a.price;
        case "popularity": return b.popularity - a.popularity;
        case "newest":
        default: return b.createdAt - a.createdAt;
      }
    });
    return list;
  }, [category, materials, priceKey, sort]);

  const shown = filtered.slice(0, visible);
  const hasMore = filtered.length > visible;

  const toggleMaterial = (m: Material) => {
    const next = materials.includes(m) ? materials.filter((x: Material) => x !== m) : [...materials, m];
    update({ materials: next });
  };

  const reset = () => {
    navigate({ search: { category: "all", sort: "newest", price: "all", materials: [], page: 1 }, replace: true });
  };

  const activeCount = (category !== "all" ? 1 : 0) + materials.length + (priceKey !== "all" ? 1 : 0);

  return (
    <>
      <Header />
      <main>
        {/* Title bar */}
        <section className="border-b border-border bg-[var(--paper)]">
          <div className="container-edge py-12 md:py-20">
            <div className="flex items-center gap-4 mb-6">
              <span className="gold-divider" />
              <span className="spec text-[10px]">Index · {products.length} pieces</span>
            </div>
            <h1 className="font-serif text-5xl md:text-7xl tracking-[-0.02em] leading-[0.95]">
              The Collection
            </h1>
            <p className="mt-6 max-w-xl text-muted-foreground text-[15px] leading-relaxed">
              Twelve essentials in 18K gold and platinum — hand-finished in our Florentine atelier.
            </p>
          </div>
        </section>

        {/* Category strip */}
        <div className="sticky top-16 md:top-20 z-30 bg-background border-b border-border">
          <div className="container-edge flex items-center justify-between gap-4 py-4 overflow-x-auto">
            <nav className="flex items-center gap-7">
              {CATEGORIES.map((c) => (
                <button
                  key={c.key}
                  onClick={() => update({ category: c.key as Category | "all" })}
                  className={`spec text-[11px] py-1 border-b transition-colors whitespace-nowrap ${
                    category === c.key ? "border-[var(--ink)] text-[var(--ink)]" : "border-transparent text-muted-foreground hover:text-[var(--ink)]"
                  }`}
                >
                  {c.label}
                </button>
              ))}
            </nav>
            <div className="flex items-center gap-3 shrink-0">
              <button
                onClick={() => setFiltersOpen(true)}
                className="lg:hidden inline-flex items-center gap-2 spec text-[10px] border border-[var(--ink)] px-3 py-2"
              >
                <SlidersHorizontal className="w-3.5 h-3.5" />
                Filters {activeCount > 0 && <span className="text-[var(--gold)]">· {activeCount}</span>}
              </button>
              <SortMenu sort={sort} onChange={(s) => update({ sort: s })} />
            </div>
          </div>
        </div>

        {/* Grid + sidebar */}
        <section className="container-edge py-12 md:py-16">
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 lg:gap-14">
            <aside className="hidden lg:block lg:col-span-3">
              <div className="sticky top-44">
                <Filters
                  materials={materials}
                  toggleMaterial={toggleMaterial}
                  priceKey={priceKey}
                  setPriceKey={(k) => update({ price: k as typeof priceKey })}
                  reset={reset}
                  activeCount={activeCount}
                />
              </div>
            </aside>

            <div className="lg:col-span-9">
              {filtered.length === 0 ? (
                <div className="py-32 text-center">
                  <p className="spec text-[11px] text-muted-foreground mb-4">No pieces match this selection</p>
                  <button onClick={reset} className="link-underline spec text-[11px]">Reset filters</button>
                </div>
              ) : (
                <>
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-x-6 gap-y-14">
                    {shown.map((p, i) => (
                      <ProductCard key={p.id} product={p} index={i} />
                    ))}
                  </div>

                  <div className="mt-20 flex flex-col items-center gap-3">
                    <span className="spec text-[10px] text-muted-foreground">
                      Showing {shown.length} of {filtered.length}
                    </span>
                    {hasMore ? (
                      <button
                        onClick={() => update({ page: page + 1 }, false)}
                        className="btn-ghost"
                      >
                        <span>Load more</span>
                        <span aria-hidden>↓</span>
                      </button>
                    ) : (
                      <span className="spec text-[10px] text-[var(--gold)]">— end of index —</span>
                    )}
                  </div>
                </>
              )}
            </div>
          </div>
        </section>

        {/* Mobile filter drawer */}
        {filtersOpen && (
          <div className="fixed inset-0 z-50 lg:hidden">
            <div className="absolute inset-0 bg-black/50" onClick={() => setFiltersOpen(false)} />
            <div className="absolute right-0 top-0 h-full w-full max-w-sm bg-background p-6 overflow-y-auto">
              <div className="flex items-center justify-between mb-8">
                <span className="spec text-[11px]">Filters</span>
                <button onClick={() => setFiltersOpen(false)} aria-label="Close"><X className="w-5 h-5" /></button>
              </div>
              <Filters
                materials={materials}
                toggleMaterial={toggleMaterial}
                priceKey={priceKey}
                setPriceKey={(k) => update({ price: k as typeof priceKey })}
                reset={reset}
                activeCount={activeCount}
              />
              <button onClick={() => setFiltersOpen(false)} className="btn-solid w-full justify-center mt-10">
                <span>Show {filtered.length} pieces</span>
              </button>
            </div>
          </div>
        )}
      </main>
      <Footer />
    </>
  );
}

function SortMenu({ sort, onChange }: { sort: SortKey; onChange: (s: SortKey) => void }) {
  const [open, setOpen] = useState(false);
  const current = SORT_OPTIONS.find((o) => o.key === sort)!;
  return (
    <div className="relative">
      <button
        onClick={() => setOpen((v) => !v)}
        className="inline-flex items-center gap-2 spec text-[10px] border border-[var(--ink)] px-3 py-2"
      >
        <span className="hidden sm:inline">Sort ·</span> {current.label}
        <ChevronDown className={`w-3 h-3 transition-transform ${open ? "rotate-180" : ""}`} />
      </button>
      {open && (
        <>
          <div className="fixed inset-0 z-30" onClick={() => setOpen(false)} />
          <div className="absolute right-0 top-full mt-1 z-40 bg-background border border-border min-w-[220px] shadow-[var(--shadow-elevated)]">
            {SORT_OPTIONS.map((o) => (
              <button
                key={o.key}
                onClick={() => { onChange(o.key); setOpen(false); }}
                className={`block w-full text-left spec text-[10px] px-4 py-3 hover:bg-[var(--paper)] ${
                  o.key === sort ? "text-[var(--gold)]" : ""
                }`}
              >
                {o.label}
              </button>
            ))}
          </div>
        </>
      )}
    </div>
  );
}

function Filters({
  materials, toggleMaterial, priceKey, setPriceKey, reset, activeCount,
}: {
  materials: Material[];
  toggleMaterial: (m: Material) => void;
  priceKey: string;
  setPriceKey: (k: string) => void;
  reset: () => void;
  activeCount: number;
}) {
  return (
    <div className="space-y-10">
      <div className="flex items-center justify-between border-t border-[var(--ink)] pt-4">
        <span className="spec text-[10px]">Refine</span>
        {activeCount > 0 && (
          <button onClick={reset} className="spec text-[10px] text-[var(--gold)] link-underline">
            Reset · {activeCount}
          </button>
        )}
      </div>

      <div>
        <p className="spec text-[10px] mb-4">Material</p>
        <ul className="space-y-3">
          {MATERIALS.map((m) => {
            const checked = materials.includes(m);
            return (
              <li key={m}>
                <label className="flex items-center gap-3 cursor-pointer group">
                  <span className={`w-4 h-4 border border-[var(--ink)] flex items-center justify-center transition-colors ${checked ? "bg-[var(--ink)]" : ""}`}>
                    {checked && <span className="w-1.5 h-1.5 bg-[var(--gold)]" />}
                  </span>
                  <input type="checkbox" checked={checked} onChange={() => toggleMaterial(m)} className="sr-only" />
                  <span className="text-sm group-hover:text-[var(--gold)] transition-colors">{m}</span>
                </label>
              </li>
            );
          })}
        </ul>
      </div>

      <div>
        <p className="spec text-[10px] mb-4">Price</p>
        <ul className="space-y-3">
          {PRICE_RANGES.map((r) => (
            <li key={r.key}>
              <label className="flex items-center gap-3 cursor-pointer group">
                <span className={`w-4 h-4 rounded-full border border-[var(--ink)] flex items-center justify-center ${priceKey === r.key ? "bg-[var(--ink)]" : ""}`}>
                  {priceKey === r.key && <span className="w-1.5 h-1.5 rounded-full bg-[var(--gold)]" />}
                </span>
                <input type="radio" name="price" checked={priceKey === r.key} onChange={() => setPriceKey(r.key)} className="sr-only" />
                <span className="text-sm group-hover:text-[var(--gold)] transition-colors">{r.label}</span>
              </label>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
