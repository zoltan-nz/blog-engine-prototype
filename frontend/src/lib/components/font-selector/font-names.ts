type Font = {
  name: string;
  family: string; // primary font name — used as identifier and for localStorage
  cssStack: string; // full CSS font-family value written to --font-family
  googleParam?: string; // present only for Google Fonts; absent for system font stacks
  letterSpacing?: string; // per-font optical correction written to --letter-spacing
};

const fonts: Font[] = [
  // ── Google Fonts ──────────────────────────────────────────────────────────
  {
    name: "Montserrat",
    family: "Montserrat",
    cssStack: '"Montserrat", sans-serif',
    googleParam: "Montserrat:ital,wght@0,100..900;1,100..900",
    letterSpacing: "-0.01em",
  },
  {
    name: "Ubuntu",
    family: "Ubuntu",
    cssStack: '"Ubuntu", sans-serif',
    googleParam:
      "Ubuntu:ital,wght@0,300;0,400;0,500;0,700;1,300;1,400;1,500;1,700",
  },
  {
    name: "Barlow Condensed",
    family: "Barlow Condensed",
    cssStack: '"Barlow Condensed", sans-serif',
    googleParam:
      "Barlow+Condensed:ital,wght@0,300;0,400;0,500;0,600;0,700;1,300;1,400;1,500;1,600;1,700",
    letterSpacing: "0.01em",
  },
  {
    name: "Bitter",
    family: "Bitter",
    cssStack: '"Bitter", serif',
    googleParam: "Bitter:ital,wght@0,100..900;1,100..900",
    letterSpacing: "0.01em",
  },
  {
    name: "Inter",
    family: "Inter",
    cssStack: '"Inter", sans-serif',
    googleParam: "Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900",
    letterSpacing: "-0.011em",
  },
  {
    name: "Geist",
    family: "Geist",
    cssStack: '"Geist", sans-serif',
    googleParam: "Geist:wght@100..900",
    letterSpacing: "-0.02em",
  },
  {
    name: "DM Sans",
    family: "DM Sans",
    cssStack: '"DM Sans", sans-serif',
    googleParam: "DM+Sans:ital,opsz,wght@0,9..40,100..1000;1,9..40,100..1000",
  },
  {
    name: "Plus Jakarta Sans",
    family: "Plus Jakarta Sans",
    cssStack: '"Plus Jakarta Sans", sans-serif',
    googleParam: "Plus+Jakarta+Sans:ital,wght@0,200..800;1,200..800",
    letterSpacing: "-0.01em",
  },
  {
    name: "Manrope",
    family: "Manrope",
    cssStack: '"Manrope", sans-serif',
    googleParam: "Manrope:wght@200..800",
    letterSpacing: "-0.01em",
  },
  {
    name: "Figtree",
    family: "Figtree",
    cssStack: '"Figtree", sans-serif',
    googleParam: "Figtree:ital,wght@0,300..900;1,300..900",
  },
  {
    name: "Outfit",
    family: "Outfit",
    cssStack: '"Outfit", sans-serif',
    googleParam: "Outfit:wght@100..900",
    letterSpacing: "-0.01em",
  },
  {
    name: "Sora",
    family: "Sora",
    cssStack: '"Sora", sans-serif',
    googleParam: "Sora:wght@100..800",
    letterSpacing: "-0.02em",
  },
  {
    name: "Nunito",
    family: "Nunito",
    cssStack: '"Nunito", sans-serif',
    googleParam: "Nunito:ital,wght@0,200..1000;1,200..1000",
  },
  {
    name: "Rubik",
    family: "Rubik",
    cssStack: '"Rubik", sans-serif',
    googleParam: "Rubik:ital,wght@0,300..900;1,300..900",
  },
  {
    name: "IBM Plex Sans",
    family: "IBM Plex Sans",
    cssStack: '"IBM Plex Sans", sans-serif',
    googleParam:
      "IBM+Plex+Sans:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;1,100;1,200;1,300;1,400;1,500;1,600;1,700",
    letterSpacing: "0.01em",
  },
  {
    name: "Source Sans 3",
    family: "Source Sans 3",
    cssStack: '"Source Sans 3", sans-serif',
    googleParam: "Source+Sans+3:ital,wght@0,200..900;1,200..900",
  },
  {
    name: "Lexend",
    family: "Lexend",
    cssStack: '"Lexend", sans-serif',
    googleParam: "Lexend:wght@100..900",
    letterSpacing: "0.01em",
  },
  {
    name: "Be Vietnam Pro",
    family: "Be Vietnam Pro",
    cssStack: '"Be Vietnam Pro", sans-serif',
    googleParam:
      "Be+Vietnam+Pro:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;0,800;1,100;1,200;1,300;1,400;1,500;1,600;1,700;1,800",
  },
  {
    name: "Poppins",
    family: "Poppins",
    cssStack: '"Poppins", sans-serif',
    googleParam:
      "Poppins:ital,wght@0,300;0,400;0,500;0,600;0,700;1,300;1,400;1,500;1,600;1,700",
    letterSpacing: "-0.01em",
  },
  {
    name: "Lato",
    family: "Lato",
    cssStack: '"Lato", sans-serif',
    googleParam: "Lato:ital,wght@0,300;0,400;0,700;1,300;1,400;1,700",
  },
];

export type { Font };
export { fonts };
