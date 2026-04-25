type Font = {
  name: string;
  family: string;
  googleParam: string;
};

const fonts: Font[] = [
  { name: 'Inter', family: 'Inter', googleParam: 'Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900' },
  { name: 'Geist', family: 'Geist', googleParam: 'Geist:wght@100..900' },
  { name: 'DM Sans', family: 'DM Sans', googleParam: 'DM+Sans:ital,opsz,wght@0,9..40,100..1000;1,9..40,100..1000' },
  { name: 'Plus Jakarta Sans', family: 'Plus Jakarta Sans', googleParam: 'Plus+Jakarta+Sans:ital,wght@0,200..800;1,200..800' },
  { name: 'Manrope', family: 'Manrope', googleParam: 'Manrope:wght@200..800' },
  { name: 'Figtree', family: 'Figtree', googleParam: 'Figtree:ital,wght@0,300..900;1,300..900' },
  { name: 'Outfit', family: 'Outfit', googleParam: 'Outfit:wght@100..900' },
  { name: 'Sora', family: 'Sora', googleParam: 'Sora:wght@100..800' },
  { name: 'Nunito', family: 'Nunito', googleParam: 'Nunito:ital,wght@0,200..1000;1,200..1000' },
  { name: 'Rubik', family: 'Rubik', googleParam: 'Rubik:ital,wght@0,300..900;1,300..900' },
  { name: 'IBM Plex Sans', family: 'IBM Plex Sans', googleParam: 'IBM+Plex+Sans:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;1,100;1,200;1,300;1,400;1,500;1,600;1,700' },
  { name: 'Source Sans 3', family: 'Source Sans 3', googleParam: 'Source+Sans+3:ital,wght@0,200..900;1,200..900' },
  { name: 'Lexend', family: 'Lexend', googleParam: 'Lexend:wght@100..900' },
  { name: 'Be Vietnam Pro', family: 'Be Vietnam Pro', googleParam: 'Be+Vietnam+Pro:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;0,800;1,100;1,200;1,300;1,400;1,500;1,600;1,700;1,800' },
  { name: 'Poppins', family: 'Poppins', googleParam: 'Poppins:ital,wght@0,300;0,400;0,500;0,600;0,700;1,300;1,400;1,500;1,600;1,700' },
  { name: 'Lato', family: 'Lato', googleParam: 'Lato:ital,wght@0,300;0,400;0,700;1,300;1,400;1,700' },
];

export type { Font };
export { fonts };
