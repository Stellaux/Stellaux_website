//! Seed local Postgres with the existing TanStack frontend's product data.
//!
//! Usage:
//!     cargo run --bin seed
//!
//! Idempotent on re-run: uses INSERT … ON CONFLICT (handle/sku) DO NOTHING
//! so re-seeding never duplicates rows. Safe to call after migrations.
//!
//! Sources:
//!   - 12 catalog products  → src/data/products.ts in the storefront
//!   - 9 craft bases + 9 accessories → src/routes/craft.tsx (BASE_ITEMS + ACCESSORIES)

use anyhow::{Context, Result};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectOptions, Database, EntityTrait, QueryFilter,
    Set,
};
use stellaux_server::entity::{
    craft_compatibility, inventory_levels, product_images, product_variants, products,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt().with_target(false).init();

    let url = std::env::var("DATABASE_URL").context("DATABASE_URL not set")?;
    let db = Database::connect(ConnectOptions::new(url)).await?;
    tracing::info!("connected — beginning seed");

    seed_catalog(&db).await?;
    let craft_handles = seed_craft(&db).await?;
    seed_compatibility(&db, &craft_handles).await?;

    tracing::info!("seed complete");
    Ok(())
}

// ─── Catalog products ───────────────────────────────────────────────────────

struct CatalogSeed {
    handle: &'static str,
    name: &'static str,
    category: &'static str,
    material: &'static str,
    price_cents: i32,
    popularity: i32,
    spec: &'static str,
    collection: &'static str,
    description: &'static str,
}

const CATALOG: &[CatalogSeed] = &[
    CatalogSeed {
        handle: "meridien-signet",
        name: "Méridien Signet",
        category: "rings",
        material: "18k Gold",
        price_cents: 42_000,
        popularity: 92,
        spec: "18K · 06μ",
        collection: "Vol. I",
        description: "An architectural signet, drawn from the lines of mid-century skylines.",
    },
    CatalogSeed {
        handle: "solitaire-filament",
        name: "Solitaire Filament",
        category: "necklaces",
        material: "Gold Vermeil",
        price_cents: 28_500,
        popularity: 85,
        spec: "0.10ct · 18K",
        collection: "Vol. I",
        description: "A single lab-grown brilliant on a 0.6mm cable chain.",
    },
    CatalogSeed {
        handle: "huggie-petite",
        name: "Huggie Petite",
        category: "earrings",
        material: "18k Gold",
        price_cents: 19_000,
        popularity: 78,
        spec: "Pair · 18K",
        collection: "Vol. I",
        description: "10mm hinged hoops engineered to disappear and reappear with the light.",
    },
    CatalogSeed {
        handle: "cable-architecte",
        name: "Cable Architecte",
        category: "bracelets",
        material: "18k Gold",
        price_cents: 34_000,
        popularity: 88,
        spec: "180mm · 18K",
        collection: "Vol. I",
        description: "Elongated cable links with a satin-machined finish.",
    },
    CatalogSeed {
        handle: "bauhaus-band",
        name: "Bauhaus Band",
        category: "rings",
        material: "18k Gold",
        price_cents: 36_000,
        popularity: 71,
        spec: "3mm · 18K",
        collection: "Vol. II",
        description: "A flat 3mm band with chamfered edges. Pure geometry.",
    },
    CatalogSeed {
        handle: "diptyque-drops",
        name: "Diptyque Drops",
        category: "earrings",
        material: "Gold Vermeil",
        price_cents: 24_000,
        popularity: 64,
        spec: "22mm · Vermeil",
        collection: "Vol. II",
        description: "Two suspended geometries — a disc above, an arc below.",
    },
    CatalogSeed {
        handle: "lineaire-chain",
        name: "Linéaire Chain",
        category: "necklaces",
        material: "18k Gold",
        price_cents: 39_500,
        popularity: 81,
        spec: "450mm · 18K",
        collection: "Vol. II",
        description: "A paperclip chain reduced to its purest expression.",
    },
    CatalogSeed {
        handle: "cuff-methode",
        name: "Cuff Méthode",
        category: "bracelets",
        material: "Sterling Silver",
        price_cents: 51_000,
        popularity: 58,
        spec: "8mm · Silver",
        collection: "Vol. II",
        description: "An open cuff in solid sterling, milled from a single ingot.",
    },
    CatalogSeed {
        handle: "pave-demi",
        name: "Pavé Demi",
        category: "rings",
        material: "Platinum",
        price_cents: 68_000,
        popularity: 89,
        spec: "Half pavé · Pt",
        collection: "Atelier",
        description: "Half-eternity pavé in platinum, set with twelve lab-grown brilliants.",
    },
    CatalogSeed {
        handle: "studio-stud",
        name: "Studio Stud",
        category: "earrings",
        material: "18k Gold",
        price_cents: 14_500,
        popularity: 73,
        spec: "4mm · 18K",
        collection: "Atelier",
        description: "A 4mm gold sphere — the quietest punctuation in the case.",
    },
    CatalogSeed {
        handle: "pendant-coordonnee",
        name: "Pendant Coordonnée",
        category: "necklaces",
        material: "Gold Vermeil",
        price_cents: 32_000,
        popularity: 67,
        spec: "Custom · Vermeil",
        collection: "Atelier",
        description: "Coordinates of a place that matters, engraved on a polished disc.",
    },
    CatalogSeed {
        handle: "tennis-demi",
        name: "Tennis Demi",
        category: "bracelets",
        material: "Platinum",
        price_cents: 95_000,
        popularity: 95,
        spec: "Half tennis · Pt",
        collection: "Atelier",
        description: "A modern half-tennis, set with thirty graduated brilliants in platinum settings.",
    },
];

/// Same set of placeholder image URLs cycled across products. Replace with
/// real CDN URLs as photography lands.
const PLACEHOLDER_IMAGES: &[&str] = &[
    "/img/product-1.jpg",
    "/img/product-2.jpg",
    "/img/product-3.jpg",
    "/img/product-4.jpg",
];

async fn seed_catalog(db: &sea_orm::DatabaseConnection) -> Result<()> {
    for (i, p) in CATALOG.iter().enumerate() {
        let product_id = upsert_product(
            db,
            p.handle,
            p.name,
            p.category,
            p.material,
            p.collection,
            p.spec,
            p.popularity,
            p.description,
            None,
            None,
        )
        .await?;

        upsert_variant(
            db,
            product_id,
            &format!("{}-default", p.handle.to_uppercase().replace('-', "_")),
            None,
            p.price_cents,
            10, // 10 g placeholder weight; refine when packaging dims are known
        )
        .await?;

        for (pos, url) in PLACEHOLDER_IMAGES.iter().enumerate() {
            insert_image(db, product_id, url, Some(p.name), pos as i32 + i as i32).await?;
        }
    }
    tracing::info!(count = CATALOG.len(), "catalog products seeded");
    Ok(())
}

// ─── Craft bases + accessories ──────────────────────────────────────────────

struct CraftSeed {
    handle: &'static str,
    name: &'static str,
    category: &'static str,
    material: &'static str,
    price_cents: i32,
    weight_grams: i32,
    role: &'static str,                       // "base" | "accessory"
    base_type: Option<&'static str>,          // pendant | chain | trunk (for bases)
    compatible_with: &'static [&'static str], // for accessories: base types it works with
}

const CRAFT: &[CraftSeed] = &[
    // ── 9 bases ─────────────────────────────────────────────────────────────
    CraftSeed {
        handle: "craft-pn-solitaire-disc",
        name: "Solitaire Disc",
        category: "necklaces",
        material: "18K Gold",
        price_cents: 18_000,
        weight_grams: 2,
        role: "base",
        base_type: Some("pendant"),
        compatible_with: &[],
    },
    CraftSeed {
        handle: "craft-pn-onyx-cabochon",
        name: "Onyx Cabochon",
        category: "necklaces",
        material: "Vermeil",
        price_cents: 24_000,
        weight_grams: 3,
        role: "base",
        base_type: Some("pendant"),
        compatible_with: &[],
    },
    CraftSeed {
        handle: "craft-pn-meridien-drop",
        name: "Méridien Drop",
        category: "necklaces",
        material: "18K Gold",
        price_cents: 29_500,
        weight_grams: 3,
        role: "base",
        base_type: Some("pendant"),
        compatible_with: &[],
    },
    CraftSeed {
        handle: "craft-ch-lineaire-18",
        name: "Linéaire 18\"",
        category: "necklaces",
        material: "18K Gold",
        price_cents: 32_000,
        weight_grams: 8,
        role: "base",
        base_type: Some("chain"),
        compatible_with: &[],
    },
    CraftSeed {
        handle: "craft-ch-cable-architecte",
        name: "Cable Architecte",
        category: "necklaces",
        material: "Platinum",
        price_cents: 38_000,
        weight_grams: 10,
        role: "base",
        base_type: Some("chain"),
        compatible_with: &[],
    },
    CraftSeed {
        handle: "craft-ch-curb-sotto",
        name: "Curb Sotto",
        category: "necklaces",
        material: "Sterling Silver",
        price_cents: 28_000,
        weight_grams: 9,
        role: "base",
        base_type: Some("chain"),
        compatible_with: &[],
    },
    CraftSeed {
        handle: "craft-tr-brass-swivel",
        name: "Brass Swivel",
        category: "trunk",
        material: "Brass",
        price_cents: 9_500,
        weight_grams: 4,
        role: "base",
        base_type: Some("trunk"),
        compatible_with: &[],
    },
    CraftSeed {
        handle: "craft-tr-walnut-toggle",
        name: "Walnut Toggle",
        category: "trunk",
        material: "Walnut + Brass",
        price_cents: 12_000,
        weight_grams: 5,
        role: "base",
        base_type: Some("trunk"),
        compatible_with: &[],
    },
    CraftSeed {
        handle: "craft-tr-karabiner-mini",
        name: "Karabiner Mini",
        category: "trunk",
        material: "Steel",
        price_cents: 8_500,
        weight_grams: 4,
        role: "base",
        base_type: Some("trunk"),
        compatible_with: &[],
    },
    // ── 9 accessories ───────────────────────────────────────────────────────
    CraftSeed {
        handle: "craft-ac-heritage-charm",
        name: "Heritage Charm",
        category: "accessory",
        material: "18K Gold",
        price_cents: 6_500,
        weight_grams: 1,
        role: "accessory",
        base_type: None,
        compatible_with: &["pendant", "chain"],
    },
    CraftSeed {
        handle: "craft-ac-gemstone-drop",
        name: "Gemstone Drop",
        category: "accessory",
        material: "18K Gold",
        price_cents: 9_500,
        weight_grams: 1,
        role: "accessory",
        base_type: None,
        compatible_with: &["pendant"],
    },
    CraftSeed {
        handle: "craft-ac-initial-bar",
        name: "Initial Bar",
        category: "accessory",
        material: "Vermeil",
        price_cents: 5_500,
        weight_grams: 1,
        role: "accessory",
        base_type: None,
        compatible_with: &["pendant", "chain"],
    },
    CraftSeed {
        handle: "craft-ac-chain-extender",
        name: "Chain Extender 2\"",
        category: "accessory",
        material: "Sterling Silver",
        price_cents: 3_500,
        weight_grams: 1,
        role: "accessory",
        base_type: None,
        compatible_with: &["chain"],
    },
    CraftSeed {
        handle: "craft-ac-id-tag",
        name: "ID Tag",
        category: "accessory",
        material: "18K Gold",
        price_cents: 4_500,
        weight_grams: 1,
        role: "accessory",
        base_type: None,
        compatible_with: &["chain"],
    },
    CraftSeed {
        handle: "craft-ac-detachable-clip",
        name: "Detachable Clip",
        category: "accessory",
        material: "Brass",
        price_cents: 3_000,
        weight_grams: 1,
        role: "accessory",
        base_type: None,
        compatible_with: &["chain", "trunk"],
    },
    CraftSeed {
        handle: "craft-ac-d-ring",
        name: "Decorative D-Ring",
        category: "accessory",
        material: "Brass",
        price_cents: 4_000,
        weight_grams: 1,
        role: "accessory",
        base_type: None,
        compatible_with: &["trunk"],
    },
    CraftSeed {
        handle: "craft-ac-leather-loop",
        name: "Leather Loop",
        category: "accessory",
        material: "Leather + Brass",
        price_cents: 5_000,
        weight_grams: 2,
        role: "accessory",
        base_type: None,
        compatible_with: &["trunk"],
    },
    CraftSeed {
        handle: "craft-ac-secondary-swivel",
        name: "Secondary Swivel",
        category: "accessory",
        material: "Steel",
        price_cents: 3_800,
        weight_grams: 1,
        role: "accessory",
        base_type: None,
        compatible_with: &["trunk"],
    },
];

/// Returns `(handle → product_id)` so the compatibility step can look up ids.
async fn seed_craft(
    db: &sea_orm::DatabaseConnection,
) -> Result<std::collections::HashMap<String, Uuid>> {
    let mut map = std::collections::HashMap::new();
    for c in CRAFT {
        let product_id = upsert_product(
            db,
            c.handle,
            c.name,
            c.category,
            c.material,
            "Craft",         // collection
            "Modular Craft", // spec
            50,              // popularity
            "Modular craft component.",
            Some(c.role),
            c.base_type,
        )
        .await?;

        let sku = format!("{}-DEFAULT", c.handle.to_uppercase().replace('-', "_"));
        upsert_variant(db, product_id, &sku, None, c.price_cents, c.weight_grams).await?;

        map.insert(c.handle.to_string(), product_id);
    }
    tracing::info!(count = CRAFT.len(), "craft components seeded");
    Ok(map)
}

// ─── Compatibility join rows ────────────────────────────────────────────────

async fn seed_compatibility(
    db: &sea_orm::DatabaseConnection,
    handles: &std::collections::HashMap<String, Uuid>,
) -> Result<()> {
    // For each accessory, find every base whose base_type appears in
    // `compatible_with`, then insert (base_id, accessory_id) pairs.
    let mut inserted = 0;
    for accessory in CRAFT.iter().filter(|c| c.role == "accessory") {
        let Some(&accessory_id) = handles.get(accessory.handle) else {
            continue;
        };
        for base in CRAFT
            .iter()
            .filter(|c| c.role == "base" && c.base_type.is_some())
        {
            let base_type = base.base_type.unwrap();
            if !accessory.compatible_with.contains(&base_type) {
                continue;
            }
            let Some(&base_id) = handles.get(base.handle) else {
                continue;
            };

            // Idempotent: check first, then insert.
            let exists = craft_compatibility::Entity::find()
                .filter(craft_compatibility::Column::BaseProductId.eq(base_id))
                .filter(craft_compatibility::Column::AccessoryProductId.eq(accessory_id))
                .one(db)
                .await?;
            if exists.is_some() {
                continue;
            }

            craft_compatibility::ActiveModel {
                base_product_id: Set(base_id),
                accessory_product_id: Set(accessory_id),
            }
            .insert(db)
            .await?;
            inserted += 1;
        }
    }
    tracing::info!(inserted, "craft compatibility rows seeded");
    Ok(())
}

// ─── Idempotent upserts ─────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
async fn upsert_product(
    db: &sea_orm::DatabaseConnection,
    handle: &str,
    name: &str,
    category: &str,
    material: &str,
    collection: &str,
    spec: &str,
    popularity: i32,
    description: &str,
    craft_role: Option<&str>,
    craft_base_type: Option<&str>,
) -> Result<Uuid> {
    if let Some(existing) = products::Entity::find()
        .filter(products::Column::Handle.eq(handle))
        .one(db)
        .await?
    {
        return Ok(existing.id);
    }

    let id = Uuid::new_v4();
    products::ActiveModel {
        id: Set(id),
        handle: Set(handle.into()),
        name: Set(name.into()),
        description: Set(Some(description.into())),
        collection: Set(Some(collection.into())),
        category: Set(category.into()),
        material: Set(material.into()),
        status: Set("active".into()),
        popularity: Set(popularity),
        spec: Set(Some(spec.into())),
        craft_role: Set(craft_role.map(Into::into)),
        craft_base_type: Set(craft_base_type.map(Into::into)),
        created_at: ActiveValue::NotSet,
        updated_at: ActiveValue::NotSet,
    }
    .insert(db)
    .await?;
    Ok(id)
}

async fn upsert_variant(
    db: &sea_orm::DatabaseConnection,
    product_id: Uuid,
    sku: &str,
    size: Option<&str>,
    price_cents: i32,
    weight_grams: i32,
) -> Result<Uuid> {
    if let Some(existing) = product_variants::Entity::find()
        .filter(product_variants::Column::Sku.eq(sku))
        .one(db)
        .await?
    {
        return Ok(existing.id);
    }

    let variant_id = Uuid::new_v4();
    product_variants::ActiveModel {
        id: Set(variant_id),
        product_id: Set(product_id),
        sku: Set(sku.into()),
        size: Set(size.map(Into::into)),
        price_cents: Set(price_cents),
        weight_grams: Set(weight_grams),
        dimensions_mm: ActiveValue::NotSet,
        created_at: ActiveValue::NotSet,
    }
    .insert(db)
    .await?;

    inventory_levels::ActiveModel {
        variant_id: Set(variant_id),
        on_hand: Set(50),
        reserved: Set(0),
        updated_at: ActiveValue::NotSet,
    }
    .insert(db)
    .await?;

    Ok(variant_id)
}

async fn insert_image(
    db: &sea_orm::DatabaseConnection,
    product_id: Uuid,
    url: &str,
    alt: Option<&str>,
    position: i32,
) -> Result<()> {
    let exists = product_images::Entity::find()
        .filter(product_images::Column::ProductId.eq(product_id))
        .filter(product_images::Column::Url.eq(url))
        .one(db)
        .await?;
    if exists.is_some() {
        return Ok(());
    }

    product_images::ActiveModel {
        id: Set(Uuid::new_v4()),
        product_id: Set(product_id),
        url: Set(url.into()),
        alt: Set(alt.map(Into::into)),
        position: Set(position),
    }
    .insert(db)
    .await?;
    Ok(())
}
