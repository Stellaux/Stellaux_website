export function Footer() {
  return (
    <footer className="bg-[var(--ink)] text-[var(--paper)] mt-24">
      <div className="h-[2px] bg-[var(--gold)]" />
      <div className="container-edge py-20 md:py-24">
        <div className="grid grid-cols-2 md:grid-cols-12 gap-10 md:gap-8">
          <div className="col-span-2 md:col-span-4">
            <h3 className="font-serif text-3xl">Maison Auré</h3>
            <p className="mt-4 text-sm text-white/60 max-w-xs leading-relaxed">
              The polished standard. Engineered jewelry for the modern professional —
              from the desk to the dinner.
            </p>
            <div className="spec text-[10px] mt-6 text-white/40">
              ISO 9001 · 18K · Conflict-free
            </div>
          </div>

          <div className="md:col-span-2 md:col-start-6">
            <p className="spec text-[10px] text-white/40 mb-5">Shop</p>
            <ul className="space-y-3 text-sm">
              <li><a className="hover:text-[var(--gold)] transition-colors" href="#">Rings</a></li>
              <li><a className="hover:text-[var(--gold)] transition-colors" href="#">Necklaces</a></li>
              <li><a className="hover:text-[var(--gold)] transition-colors" href="#">Earrings</a></li>
              <li><a className="hover:text-[var(--gold)] transition-colors" href="#">Bracelets</a></li>
            </ul>
          </div>

          <div className="md:col-span-2">
            <p className="spec text-[10px] text-white/40 mb-5">Maison</p>
            <ul className="space-y-3 text-sm">
              <li><a className="hover:text-[var(--gold)] transition-colors" href="#">Atelier</a></li>
              <li><a className="hover:text-[var(--gold)] transition-colors" href="#">Materials</a></li>
              <li><a className="hover:text-[var(--gold)] transition-colors" href="#">Journal</a></li>
              <li><a className="hover:text-[var(--gold)] transition-colors" href="#">Press</a></li>
            </ul>
          </div>

          <div className="md:col-span-2">
            <p className="spec text-[10px] text-white/40 mb-5">Service</p>
            <ul className="space-y-3 text-sm">
              <li><a className="hover:text-[var(--gold)] transition-colors" href="#">Shipping</a></li>
              <li><a className="hover:text-[var(--gold)] transition-colors" href="#">Returns</a></li>
              <li><a className="hover:text-[var(--gold)] transition-colors" href="#">Care guide</a></li>
              <li><a className="hover:text-[var(--gold)] transition-colors" href="#">Contact</a></li>
            </ul>
          </div>
        </div>

        <div className="mt-20 pt-8 border-t border-white/10 flex flex-col md:flex-row gap-4 items-start md:items-center justify-between">
          <p className="spec text-[10px] text-white/40">
            © MMXXV Maison Auré · All rights reserved
          </p>
          <p className="spec text-[10px] text-white/40">
            Designed in New York · Crafted in Florence
          </p>
        </div>
      </div>
    </footer>
  );
}
