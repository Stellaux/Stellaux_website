import { createFileRoute } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { Check, Plus, X, Gem, Link2, KeyRound } from "lucide-react";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { useCart } from "@/context/CartContext";
import p1 from "@/assets/product-1.jpg";
import p2 from "@/assets/product-2.jpg";
import p3 from "@/assets/product-3.jpg";
import p4 from "@/assets/product-4.jpg";

export const Route = createFileRoute("/craft")({
  head: () => ({
    meta: [
      { title: "Craft your piece — Maison Auré" },
      {
        name: "description",
        content:
          "Choose a base — pendant, chain, or trunk part — and build your assembly with compatible accessories.",
      },
      { property: "og:title", content: "Craft your piece — Maison Auré" },
      {
        property: "og:description",
        content: "Modular jewelry builder. Start with one. Add what fits.",
      },
    ],
  }),
  component: CraftPage,
});

type BaseType = "pendant" | "chain" | "trunk";

interface BaseItem {
  id: string;
  type: BaseType;
  name: string;
  price: number;
  img: string;
  weight: string;
  material: string;
}

interface Accessory {
  id: string;
  name: string;
  price: number;
  img: string;
  glyph: string; // tiny visual badge
  compatibleBaseTypes: BaseType[];
}

const BASE_ITEMS: BaseItem[] = [
  { id: "pn-01", type: "pendant", name: "Solitaire Disc", price: 180, img: p2, weight: "2.4g", material: "18K Gold" },
  { id: "pn-02", type: "pendant", name: "Onyx Cabochon", price: 240, img: p3, weight: "3.1g", material: "Vermeil" },
  { id: "pn-03", type: "pendant", name: "Méridien Drop", price: 295, img: p1, weight: "2.8g", material: "18K Gold" },
  { id: "ch-01", type: "chain", name: "Linéaire 18\"", price: 320, img: p4, weight: "8.2g", material: "18K Gold" },
  { id: "ch-02", type: "chain", name: "Cable Architecte", price: 380, img: p1, weight: "10.1g", material: "Platinum" },
  { id: "ch-03", type: "chain", name: "Curb Sotto", price: 280, img: p2, weight: "9.0g", material: "Sterling Silver" },
  { id: "tr-01", type: "trunk", name: "Brass Swivel", price: 95, img: p3, weight: "4.5g", material: "Brass" },
  { id: "tr-02", type: "trunk", name: "Walnut Toggle", price: 120, img: p4, weight: "5.2g", material: "Walnut + Brass" },
  { id: "tr-03", type: "trunk", name: "Karabiner Mini", price: 85, img: p1, weight: "3.9g", material: "Steel" },
];

const ACCESSORIES: Accessory[] = [
  { id: "ac-01", name: "Heritage Charm", price: 65, img: p1, glyph: "✦", compatibleBaseTypes: ["pendant", "chain"] },
  { id: "ac-02", name: "Gemstone Drop", price: 95, img: p2, glyph: "◆", compatibleBaseTypes: ["pendant"] },
  { id: "ac-03", name: "Initial Bar", price: 55, img: p3, glyph: "I", compatibleBaseTypes: ["pendant", "chain"] },
  { id: "ac-04", name: "Chain Extender 2\"", price: 35, img: p4, glyph: "+", compatibleBaseTypes: ["chain"] },
  { id: "ac-05", name: "ID Tag", price: 45, img: p1, glyph: "•", compatibleBaseTypes: ["chain"] },
  { id: "ac-06", name: "Detachable Clip", price: 30, img: p2, glyph: "↻", compatibleBaseTypes: ["chain", "trunk"] },
  { id: "ac-07", name: "Decorative D-Ring", price: 40, img: p3, glyph: "D", compatibleBaseTypes: ["trunk"] },
  { id: "ac-08", name: "Leather Loop", price: 50, img: p4, glyph: "L", compatibleBaseTypes: ["trunk"] },
  { id: "ac-09", name: "Secondary Swivel", price: 38, img: p1, glyph: "○", compatibleBaseTypes: ["trunk"] },
];

const BASE_TYPES: { key: BaseType; title: string; desc: string; Icon: React.ComponentType<{ className?: string }> }[] = [
  { key: "pendant", title: "Pendant", desc: "A worn focal point.", Icon: Gem },
  { key: "chain", title: "Chain", desc: "The connecting line.", Icon: Link2 },
  { key: "trunk", title: "Trunk Part", desc: "Hardware for bags & keys.", Icon: KeyRound },
];

function CraftPage() {
  const [baseType, setBaseType] = useState<BaseType | null>(null);
  const [baseItem, setBaseItem] = useState<BaseItem | null>(null);
  const [accessories, setAccessories] = useState<Accessory[]>([]);
  const { addItem } = useCart();

  const inventory = useMemo(
    () => (baseType ? BASE_ITEMS.filter((i) => i.type === baseType) : []),
    [baseType],
  );
  const compatible = useMemo(
    () => (baseType ? ACCESSORIES.filter((a) => a.compatibleBaseTypes.includes(baseType)) : []),
    [baseType],
  );

  const total = (baseItem?.price ?? 0) + accessories.reduce((s, a) => s + a.price, 0);

  const pickBaseType = (t: BaseType) => {
    setBaseType(t);
    setBaseItem(null);
    setAccessories([]);
  };

  const addAccessory = (a: Accessory) => {
    if (accessories.find((x) => x.id === a.id)) return;
    setAccessories([...accessories, a]);
  };
  const removeAccessory = (id: string) => setAccessories(accessories.filter((x) => x.id !== id));

  const addToCart = () => {
    if (!baseItem) return;
    addItem({
      productId: `craft-${baseItem.id}`,
      slug: "shop",
      name: `Custom ${baseItem.name}${accessories.length ? ` + ${accessories.length} accessory${accessories.length > 1 ? "s" : ""}` : ""}`,
      image: baseItem.img,
      price: total,
      size: null,
      qty: 1,
      spec: `Custom · ${baseItem.material}`,
      material: baseItem.material,
    });
  };

  return (
    <>
      <Header />
      <main>
        {/* Hero */}
        <section className="container-edge pt-16 md:pt-24 pb-10">
          <div className="flex items-center gap-4 mb-6">
            <span className="gold-divider" />
            <span className="spec text-[10px] text-[var(--gold)]">Modular Craft — Choose Your Base</span>
          </div>
          <h1 className="font-serif text-5xl md:text-7xl tracking-[-0.02em] leading-[1.02] max-w-3xl">
            Start with one.<br />
            Add what <em className="not-italic text-[var(--gold)]">fits.</em>
          </h1>
          <p className="mt-6 max-w-xl text-[15px] leading-relaxed text-muted-foreground">
            First, select a pendant, chain, or trunk part as your foundation. Then enhance it with
            compatible charms and accessories — each base unlocks its own unique add-ons.
          </p>
        </section>

        {/* Step 1 — Base type selector */}
        <section className="container-edge pb-12">
          <StepHeader n={1} title="Choose a base type" required />
          <div className="grid sm:grid-cols-3 gap-4 mt-6">
            {BASE_TYPES.map(({ key, title, desc, Icon }) => {
              const active = baseType === key;
              return (
                <button
                  key={key}
                  onClick={() => pickBaseType(key)}
                  className={`relative border p-6 md:p-8 text-left transition-all ${
                    active
                      ? "border-[var(--gold)] shadow-[var(--shadow-elevated)] bg-[var(--paper)]"
                      : "border-border hover:border-[var(--ink)]"
                  }`}
                >
                  {active && (
                    <span className="absolute top-3 right-3 w-5 h-5 rounded-full bg-[var(--gold)] text-[var(--ink)] flex items-center justify-center">
                      <Check className="w-3 h-3" strokeWidth={2.4} />
                    </span>
                  )}
                  <Icon className="w-7 h-7 mb-6" strokeWidth={1.2} />
                  <div className="font-serif text-3xl mb-1">{title}</div>
                  <p className="spec text-[10px] text-muted-foreground">{desc}</p>
                </button>
              );
            })}
          </div>
        </section>

        {/* Step 2 — Two-column build area */}
        <section className="container-edge pb-12">
          <StepHeader
            n={2}
            title={baseType ? `Choose your ${BASE_TYPES.find((b) => b.key === baseType)!.title.toLowerCase()}` : "Choose a base item"}
            disabled={!baseType}
          />
          <div className={`mt-6 grid lg:grid-cols-[1.1fr_1fr] gap-6 lg:gap-10 ${!baseType ? "opacity-40 pointer-events-none" : ""}`}>
            {/* Inventory */}
            <div>
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {inventory.map((it) => {
                  const active = baseItem?.id === it.id;
                  return (
                    <button
                      key={it.id}
                      onClick={() => setBaseItem(it)}
                      className={`text-left border transition-all ${
                        active ? "border-[var(--gold)]" : "border-border hover:border-[var(--ink)]"
                      }`}
                    >
                      <div className="aspect-square overflow-hidden bg-[var(--paper)]">
                        <img src={it.img} alt={it.name} className="w-full h-full object-cover" />
                      </div>
                      <div className="p-3">
                        <div className="font-serif text-[15px] leading-tight">{it.name}</div>
                        <div className="mt-1 flex items-center justify-between">
                          <span className="spec text-[8px] text-muted-foreground">{it.weight}</span>
                          <span className="font-mono text-[11px] tabular-nums text-[var(--gold)]">${it.price}</span>
                        </div>
                      </div>
                    </button>
                  );
                })}
              </div>

              {/* Accessories */}
              {baseType && (
                <div className="mt-10">
                  <div className="flex items-center gap-3 mb-4">
                    <span className="spec text-[10px] text-[var(--gold)]">
                      Compatible Accessories for {BASE_TYPES.find((b) => b.key === baseType)!.title}
                    </span>
                    <span className="hairline flex-1" />
                  </div>
                  <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                    {compatible.map((a) => {
                      const added = !!accessories.find((x) => x.id === a.id);
                      return (
                        <div key={a.id} className="border border-border">
                          <div className="aspect-square overflow-hidden bg-[var(--paper)]">
                            <img src={a.img} alt={a.name} className="w-full h-full object-cover" />
                          </div>
                          <div className="p-3 flex items-center justify-between gap-2">
                            <div className="min-w-0">
                              <div className="font-serif text-[14px] leading-tight truncate">{a.name}</div>
                              <div className="font-mono text-[10px] text-muted-foreground tabular-nums">${a.price}</div>
                            </div>
                            <button
                              onClick={() => (added ? removeAccessory(a.id) : addAccessory(a))}
                              className={`shrink-0 w-8 h-8 inline-flex items-center justify-center border transition-colors ${
                                added
                                  ? "border-[var(--gold)] text-[var(--gold)]"
                                  : "border-border hover:border-[var(--ink)]"
                              }`}
                              aria-label={added ? `Remove ${a.name}` : `Add ${a.name}`}
                            >
                              {added ? <Check className="w-3 h-3" /> : <Plus className="w-3 h-3" />}
                            </button>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}
            </div>

            {/* Visual build display */}
            <aside className="lg:sticky lg:top-24 self-start">
              <div className="border border-border bg-[var(--paper)] p-6">
                <div className="spec text-[9px] text-muted-foreground mb-4">Your Assembly</div>

                {/* Visual canvas */}
                <div className="relative bg-background border border-border min-h-[260px] flex flex-col items-center justify-center gap-6 py-8 px-6">
                  {!baseItem ? (
                    <p className="spec text-[10px] text-muted-foreground text-center">
                      Select a base item to begin
                    </p>
                  ) : (
                    <>
                      <div className="relative">
                        <img
                          src={baseItem.img}
                          alt={baseItem.name}
                          className="w-32 h-32 object-cover"
                        />
                        <span className="absolute -top-2 -left-2 spec text-[8px] text-[var(--gold)] bg-background px-1">
                          base
                        </span>
                      </div>

                      {accessories.length > 0 && (
                        <>
                          <div className="hairline w-12" style={{ borderTop: "1px dashed var(--border)", height: 0 }} />
                          <div className="flex flex-wrap items-center justify-center gap-2 max-w-[260px]">
                            {accessories.map((a) => (
                              <button
                                key={a.id}
                                onClick={() => removeAccessory(a.id)}
                                className="group relative w-9 h-9 border border-[var(--gold)] flex items-center justify-center hover:bg-[var(--ink)] hover:text-[var(--paper)] transition-colors"
                                title={`Remove ${a.name}`}
                              >
                                <span className="font-serif text-base">{a.glyph}</span>
                                <X className="absolute -top-1.5 -right-1.5 w-3 h-3 bg-background border border-border opacity-0 group-hover:opacity-100" />
                              </button>
                            ))}
                          </div>
                        </>
                      )}
                    </>
                  )}
                </div>

                {/* Summary */}
                {baseItem && (
                  <div className="mt-5 space-y-3">
                    <Row label="Base" value={baseItem.name} sub={`$${baseItem.price}`} />
                    {accessories.map((a) => (
                      <Row key={a.id} label="Accessory" value={a.name} sub={`$${a.price}`} onRemove={() => removeAccessory(a.id)} />
                    ))}
                  </div>
                )}

                <div className="hairline my-5" />
                <div className="flex items-baseline justify-between">
                  <span className="spec text-[10px]">Total</span>
                  <span className="font-mono text-2xl tabular-nums text-[var(--gold)]">
                    ${total.toLocaleString()}
                  </span>
                </div>
                {baseItem && accessories.some(() => false) /* placeholder for warnings */}

                <button
                  onClick={addToCart}
                  disabled={!baseItem}
                  className="btn-solid w-full justify-center mt-6 disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  <span>Add assembly to bag</span>
                  <span aria-hidden>→</span>
                </button>
                <p className="spec text-[9px] text-muted-foreground mt-4 text-center">
                  Hand-finished in Florence · 4–6 week lead time
                </p>
              </div>
            </aside>
          </div>
        </section>
      </main>
      <Footer />
    </>
  );
}

function StepHeader({ n, title, required, disabled }: { n: number; title: string; required?: boolean; disabled?: boolean }) {
  return (
    <div className={`flex items-center gap-4 ${disabled ? "opacity-40" : ""}`}>
      <span className="font-mono text-xs tabular-nums">0{n}</span>
      <span className="hairline w-8" />
      <span className="spec text-[10px]">{title}</span>
      {required && <span className="spec text-[9px] text-[var(--gold)]">Required</span>}
    </div>
  );
}

function Row({
  label,
  value,
  sub,
  onRemove,
}: {
  label: string;
  value: string;
  sub: string;
  onRemove?: () => void;
}) {
  return (
    <div className="flex items-center justify-between gap-3">
      <div className="min-w-0">
        <div className="spec text-[8px] text-muted-foreground">{label}</div>
        <div className="font-serif text-base leading-tight truncate">{value}</div>
      </div>
      <div className="flex items-center gap-3 shrink-0">
        <span className="font-mono text-xs tabular-nums">{sub}</span>
        {onRemove && (
          <button
            onClick={onRemove}
            className="w-6 h-6 inline-flex items-center justify-center border border-border hover:border-[var(--ink)]"
            aria-label="Remove"
          >
            <X className="w-3 h-3" />
          </button>
        )}
      </div>
    </div>
  );
}
