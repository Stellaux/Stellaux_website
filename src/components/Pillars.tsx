const pillars = [
  {
    n: "I",
    t: "Refined Utility",
    d: "Designed to be worn under a cuff, over a turtleneck, into the boardroom and out to dinner.",
  },
  {
    n: "II",
    t: "Engineered Plating",
    d: "Six-micron 18K gold over recycled brass — five times the industry standard, certified for life.",
  },
  {
    n: "III",
    t: "Quiet Provenance",
    d: "Hand-finished in a single Florentine atelier. No outsourcing, no shortcuts, no spectacle.",
  },
];

export function Pillars() {
  return (
    <section className="py-24 md:py-32 bg-[var(--paper)] border-y border-border">
      <div className="container-edge">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-12 md:gap-16">
          {pillars.map((p) => (
            <article key={p.n} className="border-t border-[var(--ink)] pt-6">
              <div className="flex items-baseline justify-between mb-6">
                <span className="font-serif text-5xl">{p.n}</span>
                <span className="spec text-[10px] text-muted-foreground">Principle</span>
              </div>
              <h3 className="font-serif text-2xl md:text-3xl mb-4">{p.t}</h3>
              <p className="text-sm leading-relaxed text-muted-foreground">{p.d}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}
