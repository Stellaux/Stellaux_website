import craft from "@/assets/craftsmanship.jpg";

export function Craft() {
  return (
    <section
      id="craft"
      className="relative py-24 md:py-40 overflow-hidden"
      style={{
        backgroundImage: `linear-gradient(to right, rgba(30,30,30,0.92) 0%, rgba(30,30,30,0.55) 60%, rgba(30,30,30,0.2) 100%), url(${craft})`,
        backgroundSize: "cover",
        backgroundPosition: "center",
      }}
    >
      <div className="container-edge text-[var(--paper)]">
        <div className="max-w-xl">
          <div className="flex items-center gap-4 mb-6">
            <span className="gold-divider" />
            <span className="spec text-[10px] text-white/60">Atelier — Florence</span>
          </div>
          <h2 className="font-serif text-4xl md:text-6xl tracking-[-0.02em] leading-[1.05]">
            Drawn by hand.<br />
            Finished to the <em className="not-italic text-[var(--gold)]">micron.</em>
          </h2>
          <p className="mt-8 text-[15px] leading-relaxed text-white/70">
            Every piece begins on a walnut bench in Oltrarno — sketched in graphite,
            measured by caliper, and plated to a six-micron gold standard. The result
            is jewelry that reads as quiet on the skin, and unmistakable in person.
          </p>

          <div className="mt-12 grid grid-cols-3 gap-6 max-w-md border-t border-white/15 pt-8">
            <SpecRow k="Plating" v="06 μm" />
            <SpecRow k="Alloy" v="18 KT" />
            <SpecRow k="Origin" v="IT 🇮🇹" />
          </div>

          <a href="#" className="inline-block mt-10 spec text-[11px] text-[var(--gold)] link-underline">
            Read the dossier →
          </a>
        </div>
      </div>
    </section>
  );
}

function SpecRow({ k, v }: { k: string; v: string }) {
  return (
    <div>
      <div className="spec text-[9px] text-white/40">{k}</div>
      <div className="font-mono text-sm mt-1">{v}</div>
    </div>
  );
}
