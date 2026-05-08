import { Link } from "@tanstack/react-router";
import { Search, ShoppingBag, User, Menu } from "lucide-react";

export function Header() {
  return (
    <header className="sticky top-0 z-50 bg-background/90 backdrop-blur-md border-b border-border">
      <div className="container-edge flex items-center justify-between h-16 md:h-20">
        <button className="md:hidden p-2 -ml-2" aria-label="Menu">
          <Menu className="w-5 h-5" />
        </button>

        <nav className="hidden md:flex items-center gap-10">
          <Link to="/shop" className="nav-link">Shop</Link>
          <a href="#craft" className="nav-link">Craft</a>
          <a href="#journal" className="nav-link">Journal</a>
          <span
            className="nav-link opacity-40 cursor-not-allowed"
            title="Coming Q4"
          >
            Visual Lab
          </span>
        </nav>

        <Link to="/" className="absolute left-1/2 -translate-x-1/2 flex flex-col items-center">
          <span className="font-serif text-2xl md:text-[28px] tracking-tight leading-none">
            Maison Auré
          </span>
          <span className="spec text-[9px] mt-1 text-muted-foreground">Est. MMXXV</span>
        </Link>

        <div className="flex items-center gap-2 md:gap-5">
          <button aria-label="Search" className="p-2 hover:text-[var(--gold)] transition-colors">
            <Search className="w-[18px] h-[18px]" strokeWidth={1.4} />
          </button>
          <button aria-label="Account" className="p-2 hover:text-[var(--gold)] transition-colors hidden md:inline-flex">
            <User className="w-[18px] h-[18px]" strokeWidth={1.4} />
          </button>
          <button aria-label="Cart" className="p-2 hover:text-[var(--gold)] transition-colors relative">
            <ShoppingBag className="w-[18px] h-[18px]" strokeWidth={1.4} />
            <span className="absolute -top-0.5 -right-0.5 w-4 h-4 text-[10px] flex items-center justify-center bg-[var(--ink)] text-[var(--paper)] font-mono">
              0
            </span>
          </button>
        </div>
      </div>
    </header>
  );
}
