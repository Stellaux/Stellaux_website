import { Link } from "@tanstack/react-router";
import { products } from "@/data/products";
import { ProductCard } from "@/components/ProductCard";

export function FeaturedGrid() {
  const featured = products.slice(0, 4);
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
          <Link to="/shop" className="hidden md:inline-flex link-underline spec text-[11px] pb-1">
            View all {products.length} pieces
          </Link>
        </div>

        <div className="grid grid-cols-2 lg:grid-cols-4 gap-x-6 gap-y-14">
          {featured.map((p, i) => (
            <ProductCard key={p.id} product={p} index={i} />
          ))}
        </div>

        <div className="mt-12 md:hidden">
          <Link to="/shop" className="link-underline spec text-[11px]">
            View all {products.length} pieces
          </Link>
        </div>
      </div>
    </section>
  );
}
