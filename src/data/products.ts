import p1 from "@/assets/product-1.jpg";
import p2 from "@/assets/product-2.jpg";
import p3 from "@/assets/product-3.jpg";
import p4 from "@/assets/product-4.jpg";

export type Category = "rings" | "necklaces" | "earrings" | "bracelets";
export type Material = "18k Gold" | "Gold Vermeil" | "Sterling Silver" | "Platinum";

export interface Product {
  id: string;
  slug: string;
  name: string;
  category: Category;
  price: number;
  material: Material;
  popularity: number; // higher = more popular
  createdAt: number; // higher = newer
  images: string[];
  spec: string;
  description: string;
  details: string[];
  collection: string;
}

const IMG = [p1, p2, p3, p4];

const seed: Omit<Product, "id" | "slug" | "images">[] = [
  { name: "Méridien Signet", category: "rings", price: 420, material: "18k Gold", popularity: 92, createdAt: 12, spec: "18K · 06μ", description: "An architectural signet, drawn from the lines of mid-century skylines. Worn alone or stacked.", details: ["18K gold over recycled brass · 6 micron plating", "Hand-finished in Florence", "Comes in sizes 5–10", "Lifetime polish & re-plating service"], collection: "Vol. I" },
  { name: "Solitaire Filament", category: "necklaces", price: 285, material: "Gold Vermeil", popularity: 85, createdAt: 11, spec: "0.10ct · 18K", description: "A single lab-grown brilliant on a 0.6mm cable chain — the quiet punctuation to any neckline.", details: ["0.10ct lab-grown diamond, VS clarity", "16/18 inch adjustable chain", "Gold vermeil over 925 silver", "Spring-ring clasp"], collection: "Vol. I" },
  { name: "Huggie Petite", category: "earrings", price: 190, material: "18k Gold", popularity: 78, createdAt: 10, spec: "Pair · 18K", description: "10mm hinged hoops engineered to disappear and reappear with the light.", details: ["10mm outer diameter", "18K gold, hinged closure", "Hypoallergenic post", "Sold as a pair"], collection: "Vol. I" },
  { name: "Cable Architecte", category: "bracelets", price: 340, material: "18k Gold", popularity: 88, createdAt: 9, spec: "180mm · 18K", description: "Elongated cable links with a satin-machined finish. Reads as fluid, weighs in at exactly 12g.", details: ["180mm length, lobster clasp", "18K gold plating · 6μ", "Gross weight: 12g", "Adjustable to 165mm"], collection: "Vol. I" },
  { name: "Bauhaus Band", category: "rings", price: 360, material: "18k Gold", popularity: 71, createdAt: 8, spec: "3mm · 18K", description: "A flat 3mm band with chamfered edges. Pure geometry.", details: ["3mm wide, comfort-fit interior", "18K gold · 6μ plating", "Sizes 4–12", "Engravable inner band"], collection: "Vol. II" },
  { name: "Diptyque Drops", category: "earrings", price: 240, material: "Gold Vermeil", popularity: 64, createdAt: 7, spec: "22mm · Vermeil", description: "Two suspended geometries — a disc above, an arc below.", details: ["22mm drop length", "Gold vermeil over 925 silver", "Posts with secure backings", "Sold as a pair"], collection: "Vol. II" },
  { name: "Linéaire Chain", category: "necklaces", price: 395, material: "18k Gold", popularity: 81, createdAt: 6, spec: "450mm · 18K", description: "A paperclip chain reduced to its purest expression. Equally at ease layered or alone.", details: ["18 inch length", "2.5mm link width", "18K gold · 6μ plating", "Spring-ring clasp"], collection: "Vol. II" },
  { name: "Cuff Méthode", category: "bracelets", price: 510, material: "Sterling Silver", popularity: 58, createdAt: 5, spec: "8mm · Silver", description: "An open cuff in solid sterling, milled to exactness from a single ingot.", details: ["8mm bandwidth", "925 sterling silver", "Adjustable opening", "Anti-tarnish finish"], collection: "Vol. II" },
  { name: "Pavé Demi", category: "rings", price: 680, material: "Platinum", popularity: 89, createdAt: 4, spec: "Half pavé · Pt", description: "Half-eternity pavé in platinum, set with twelve lab-grown brilliants.", details: ["12 × 0.02ct lab-grown diamonds", "950 Platinum", "Sizes 4–9", "Comes with certificate"], collection: "Atelier" },
  { name: "Studio Stud", category: "earrings", price: 145, material: "18k Gold", popularity: 73, createdAt: 3, spec: "4mm · 18K", description: "A 4mm gold sphere — the quietest punctuation in the case.", details: ["4mm sphere diameter", "18K gold · 6μ plating", "Hypoallergenic post", "Sold as a pair"], collection: "Atelier" },
  { name: "Pendant Coordonnée", category: "necklaces", price: 320, material: "Gold Vermeil", popularity: 67, createdAt: 2, spec: "Custom · Vermeil", description: "Coordinates of a place that matters, engraved on a polished disc.", details: ["20mm disc diameter", "Gold vermeil over 925 silver", "Custom engraving included", "16/18 inch adjustable chain"], collection: "Atelier" },
  { name: "Tennis Demi", category: "bracelets", price: 950, material: "Platinum", popularity: 95, createdAt: 1, spec: "Half tennis · Pt", description: "A modern half-tennis, set with thirty graduated brilliants in platinum settings.", details: ["30 × 0.03ct lab-grown diamonds", "950 Platinum", "180mm length", "Box clasp with safety"], collection: "Atelier" },
];

export const products: Product[] = seed.map((p, i) => {
  const slug = p.name
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
  const cover = IMG[i % IMG.length];
  const images = [cover, IMG[(i + 1) % IMG.length], IMG[(i + 2) % IMG.length], IMG[(i + 3) % IMG.length]];
  return { id: String(i + 1).padStart(3, "0"), slug, images, ...p };
});

export const getProductBySlug = (slug: string) => products.find((p) => p.slug === slug);
export const relatedProducts = (p: Product) =>
  products.filter((x) => x.collection === p.collection && x.id !== p.id).slice(0, 4);

export const CATEGORIES: { key: Category | "all"; label: string }[] = [
  { key: "all", label: "All" },
  { key: "rings", label: "Rings" },
  { key: "necklaces", label: "Necklaces" },
  { key: "earrings", label: "Earrings" },
  { key: "bracelets", label: "Bracelets" },
];

export const MATERIALS: Material[] = ["18k Gold", "Gold Vermeil", "Sterling Silver", "Platinum"];
