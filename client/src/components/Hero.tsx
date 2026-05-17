import heroImg from "@/assets/hero.jpg";
import woodImg from "@/assets/wood.jpg";

export function Hero() {
  return (
    <section className="relative bg-[var(--paper)]">
      <div className="container-edge pt-12 md:pt-20 pb-16 md:pb-24">
        <div className="grid grid-cols-1 md:grid-cols-12 gap-10 md:gap-16 items-end">
          <div className="md:col-span-6 md:pb-8">
            <div className="flex items-center gap-4 mb-8 reveal">
              <span className="gold-divider" />
              <span className="spec text-[10px]">Vol. I — The Polished Standard</span>
            </div>

            <h1 className="font-serif text-[44px] sm:text-[64px] md:text-[88px] leading-[0.95] tracking-[-0.02em] reveal reveal-delay-1">
              Engineered<br />
              for the hours<br />
              <em className="not-italic text-[var(--gold)]">in between.</em>
            </h1>

            <p className="mt-8 max-w-md text-[15px] leading-relaxed text-muted-foreground reveal reveal-delay-2">
              Adornment for the modern professional — precise, weightless, and quietly
              extraordinary. From the morning desk to the evening table.
            </p>

            <div className="mt-10 flex flex-wrap items-center gap-4 reveal reveal-delay-3">
              <button className="btn-solid">
                <span>Shop the collection</span>
                <span aria-hidden>→</span>
              </button>
              <a href="#craft" className="link-underline spec text-[11px]">
                Read the dossier
              </a>
            </div>

            <div className="mt-16 grid grid-cols-3 gap-6 max-w-md">
              <Stat label="Microns plating" value="06" />
              <Stat label="Karats" value="18" />
              <Stat label="Year warranty" value="∞" />
            </div>
          </div>

          <div className="md:col-span-6 relative">
            <div className="relative aspect-[3/4] overflow-hidden">
              <img
                src={heroImg}
                alt="Editorial portrait wearing Maison Auré jewelry"
                width={1080}
                height={1440}
                className="w-full h-full object-cover"
              />
              <div className="absolute top-0 right-0 spec text-[10px] text-white/80 p-4 mix-blend-difference">
                Fig. 01 / Edition 001
              </div>
            </div>

            {/* Wood accent tile */}
            <div
              className="absolute -bottom-6 -left-6 w-28 h-28 md:w-40 md:h-40 hidden sm:block shadow-[var(--shadow-elevated)]"
              style={{
                backgroundImage: `url(${woodImg})`,
                backgroundSize: "cover",
                backgroundPosition: "center",
              }}
              aria-hidden
            >
              <div className="w-full h-full bg-black/15 flex items-end p-3">
                <span className="spec text-[9px] text-white">Atelier · Walnut</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Marquee strip */}
      <div className="border-y border-border py-5 overflow-hidden">
        <div className="marquee-track spec text-[11px] text-muted-foreground">
          {Array.from({ length: 2 }).map((_, i) => (
            <div key={i} className="flex gap-16 shrink-0">
              <span>Free worldwide shipping ⟶ orders over $200</span>
              <span className="text-[var(--gold)]">✦</span>
              <span>Lifetime polish &amp; service</span>
              <span className="text-[var(--gold)]">✦</span>
              <span>18K gold, hand-finished in Florence</span>
              <span className="text-[var(--gold)]">✦</span>
              <span>Conflict-free · Recycled metals</span>
              <span className="text-[var(--gold)]">✦</span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <div className="font-serif text-3xl">{value}</div>
      <div className="spec text-[9px] text-muted-foreground mt-1">{label}</div>
    </div>
  );
}
