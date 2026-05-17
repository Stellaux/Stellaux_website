import { createFileRoute } from "@tanstack/react-router";
import { Header } from "@/components/Header";
import { Footer } from "@/components/Footer";
import { Hero } from "@/components/Hero";
import { FeaturedGrid } from "@/components/FeaturedGrid";
import { Craft } from "@/components/Craft";
import { Pillars } from "@/components/Pillars";
import { Newsletter } from "@/components/Newsletter";

export const Route = createFileRoute("/")({
  head: () => ({
    meta: [
      { title: "Maison Auré — The Polished Standard" },
      {
        name: "description",
        content:
          "Engineered jewelry for the modern professional. 18K gold, hand-finished in Florence — from the desk to the dinner.",
      },
      { property: "og:title", content: "Maison Auré — The Polished Standard" },
      {
        property: "og:description",
        content:
          "Engineered jewelry for the modern professional. 18K gold, hand-finished in Florence.",
      },
    ],
  }),
  component: Index,
});

function Index() {
  return (
    <>
      <Header />
      <main>
        <Hero />
        <FeaturedGrid />
        <Craft />
        <Pillars />
        <Newsletter />
      </main>
      <Footer />
    </>
  );
}
