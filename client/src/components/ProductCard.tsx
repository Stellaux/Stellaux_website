import { Link } from "@tanstack/react-router";
import type { Product } from "@/data/products";
import { Heart, ShoppingBag } from "lucide-react";

export function ProductCard({ product, index }: { product: Product; index: number }) {
  return (
    <Link
      to="/shop/$slug"
      params={{ slug: product.slug }}
      className="product-card group"
    >
      <div className="product-image relative">
        <img src={product.images[0]} alt={product.name} loading="lazy" width={1024} height={1280} />
        <span className="absolute top-3 left-3 spec text-[9px] bg-white/90 px-2 py-1">
          No. {String(index + 1).padStart(3, "0")}
        </span>
        <span className="absolute top-3 right-3 spec text-[9px] text-muted-foreground bg-white/90 px-2 py-1">
          {product.spec}
        </span>

        <div className="absolute bottom-3 right-3 flex flex-col gap-2 opacity-0 translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-500">
          <button
            aria-label="Add to wishlist"
            onClick={(e) => { e.preventDefault(); }}
            className="w-9 h-9 bg-white/95 hover:bg-[var(--ink)] hover:text-white flex items-center justify-center transition-colors"
          >
            <Heart className="w-4 h-4" strokeWidth={1.4} />
          </button>
          <button
            aria-label="Quick add to cart"
            onClick={(e) => { e.preventDefault(); }}
            className="w-9 h-9 bg-white/95 hover:bg-[var(--ink)] hover:text-white flex items-center justify-center transition-colors"
          >
            <ShoppingBag className="w-4 h-4" strokeWidth={1.4} />
          </button>
        </div>
      </div>
      <div className="pt-5 flex items-start justify-between gap-3">
        <div>
          <p className="spec text-[9px] text-muted-foreground mb-1 capitalize">{product.category.slice(0, -1)}</p>
          <h3 className="font-serif text-xl leading-tight">{product.name}</h3>
        </div>
        <p className="font-mono text-sm text-[var(--gold)] tabular-nums whitespace-nowrap">
          ${product.price}
        </p>
      </div>
    </Link>
  );
}
