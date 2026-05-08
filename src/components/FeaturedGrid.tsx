import p1 from "@/assets/product-1.jpg";
import p2 from "@/assets/product-2.jpg";
import p3 from "@/assets/product-3.jpg";
import p4 from "@/assets/product-4.jpg";

const products = [
  { id: "01", name: "Méridien Signet", category: "Ring", price: "$420", img: p1, spec: "18K · 06μ" },
  { id: "02", name: "Solitaire Filament", category: "Necklace", price: "$285", img: p2, spec: "0.10ct · 18K" },
  { id: "03", name: "Huggie Petite", category: "Earrings", price: "$190", img: p3, spec: "Pair · 18K" },
  { id: "04", name: "Cable Architecte", category: "Bracelet", price: "$340", img: p4, spec: "180mm · 18K" },
];

export function FeaturedGrid() {
  return (
    <section className="py-24 md:py-32 bg-background" id="shop">
      <div className="container-edge">
        <div className="flex items-end justify-between gap-8 mb-14 md:mb-20">
          <div>
            <div className="flex items-center gap-4 mb-5">
              <span className="gold-divider" />
              <span className="spec text-[10px]">New collection / 001</span>
            </div>
            <h2 className="font-serif text-4xl md:text-6xl tracking-[-0.02em] max-w-2xl">
              Pieces for the everyday <em className="not-italic">extraordinary.</em>
            </h2>
          </div>
          <a href="#" className="hidden md:inline-flex link-underline spec text-[11px] pb-1">
            View all 24 pieces
          </a>
        </div>

        <div className="grid grid-cols-2 lg:grid-cols-4 gap-x-6 gap-y-14">
          {products.map((p, i) => (
            <a key={p.id} href="#" className="product-card group">
              <div className="product-image relative">
                <img
                  src={p.img}
                  alt={p.name}
                  loading="lazy"
                  width={1024}
                  height={1280}
                />
                <span className="absolute top-3 left-3 spec text-[9px] bg-white/90 px-2 py-1">
                  No. {String(i + 1).padStart(3, "0")}
                </span>
                <span className="absolute top-3 right-3 spec text-[9px] text-muted-foreground bg-white/90 px-2 py-1">
                  {p.spec}
                </span>
              </div>
              <div className="pt-5 flex items-start justify-between gap-3">
                <div>
                  <p className="spec text-[9px] text-muted-foreground mb-1">{p.category}</p>
                  <h3 className="font-serif text-xl leading-tight">{p.name}</h3>
                </div>
                <p className="font-mono text-sm text-[var(--gold)] tabular-nums whitespace-nowrap">
                  {p.price}
                </p>
              </div>
            </a>
          ))}
        </div>

        <div className="mt-12 md:hidden">
          <a href="#" className="link-underline spec text-[11px]">View all 24 pieces</a>
        </div>
      </div>
    </section>
  );
}
